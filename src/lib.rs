//! Tree Parser Library
//!
//! A comprehensive solution for parsing and searching code elements across multiple programming languages.
//! Uses tree-sitter as a backend and supports finding functions, classes, structs, interfaces, and other code constructs.

use std::collections::HashMap;
use std::path::Path;
use std::sync::Arc;

use serde::{Deserialize, Serialize};
use thiserror::Error;
use tree_sitter::Tree;

// Re-export commonly used types
pub use tree_sitter::{Point, Range};

// Language modules
mod languages;
pub use languages::*;

// Error types
#[derive(Error, Debug, Clone, Serialize, Deserialize)]
pub enum Error {
    #[error("IO error: {0}")]
    Io(String),
    #[error("Parse error: {0}")]
    Parse(String),
    #[error("Unsupported language: {0}")]
    UnsupportedLanguage(String),
    #[error("File too large: {0} bytes")]
    FileTooLarge(usize),
    #[error("Permission denied: {0}")]
    PermissionDenied(String),
    #[error("Invalid query: {0}")]
    InvalidQuery(String),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ErrorType {
    ParseError,
    IoError,
    UnsupportedLanguage,
    FileTooLarge,
    PermissionDenied,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileError {
    pub file_path: String,
    pub error_type: ErrorType,
    pub message: String,
}

// Core data structures
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Language {
    Python,
    Rust,
    JavaScript,
    TypeScript,
    Java,
    C,
    Cpp,
    Go,
    CSharp,
    Php,
    Ruby,
    Swift,
    Kotlin,
    Scala,
    Haskell,
    Lua,
    Perl,
    R,
    Bash,
    PowerShell,
    Html,
    Css,
    Sql,
    Json,
    Yaml,
    Toml,
    Xml,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum LanguageDetection {
    ByExtension,
    ByContent,
    ByShebang,
    Combined, // Uses all methods with fallback priority
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Parameter {
    pub name: String,
    pub param_type: Option<String>,
    pub default_value: Option<String>,
    pub is_variadic: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConstructMetadata {
    pub visibility: Option<String>,
    pub modifiers: Vec<String>,
    pub parameters: Vec<Parameter>,
    pub return_type: Option<String>,
    pub inheritance: Vec<String>,
    pub annotations: Vec<String>,
    pub documentation: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CodeConstruct {
    pub node_type: String,
    pub name: Option<String>,
    pub source_code: String,
    pub start_line: usize,
    pub end_line: usize,
    pub start_byte: usize,
    pub end_byte: usize,
    pub parent: Option<Box<CodeConstruct>>,
    pub children: Vec<CodeConstruct>,
    pub metadata: ConstructMetadata,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ParsedFile {
    pub file_path: String,
    pub relative_path: String,
    pub language: Language,
    pub constructs: Vec<CodeConstruct>,
    #[serde(skip)]
    pub syntax_tree: Option<Tree>,
    pub file_size_bytes: usize,
    pub parse_time_ms: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ParsedProject {
    pub root_path: String,
    pub files: Vec<ParsedFile>,
    pub total_files_processed: usize,
    pub processing_time_ms: u64,
    pub language_distribution: HashMap<Language, usize>,
    pub error_files: Vec<FileError>,
}

#[derive(Clone)]
pub struct FileFilter {
    pub extensions: Option<Vec<String>>,
    pub languages: Option<Vec<Language>>,
    pub min_size_bytes: Option<usize>,
    pub max_size_bytes: Option<usize>,
    pub custom_predicate: Option<Arc<dyn Fn(&Path) -> bool + Send + Sync>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ParseOptions {
    pub max_concurrent_files: usize,
    pub include_hidden_files: bool,
    pub max_file_size_mb: usize,
    pub recursive: bool,
    pub ignore_patterns: Vec<String>,
    pub language_detection: LanguageDetection,
    pub enable_caching: bool,
    pub thread_pool_size: Option<usize>,
}

impl Default for ParseOptions {
    fn default() -> Self {
        Self {
            max_concurrent_files: num_cpus::get() * 2,
            include_hidden_files: false,
            max_file_size_mb: 10,
            recursive: true,
            ignore_patterns: vec![
                "node_modules".to_string(),
                ".git".to_string(),
                "target".to_string(),
                "build".to_string(),
            ],
            language_detection: LanguageDetection::ByExtension,
            enable_caching: true,
            thread_pool_size: None, // Uses rayon's default
        }
    }
}

// Core API functions will be implemented in separate modules
mod parser;
mod search;
mod utils;

pub use parser::*;
pub use search::*;
pub use utils::*;
// pub use test_compile::*; // Commented out as not currently used

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_language_detection() {
        assert_eq!(detect_language_by_extension("test.py"), Some(Language::Python));
        assert_eq!(detect_language_by_extension("test.rs"), Some(Language::Rust));
        assert_eq!(detect_language_by_extension("test.js"), Some(Language::JavaScript));
    }
}
