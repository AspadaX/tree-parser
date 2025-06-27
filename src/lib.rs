//! # Tree Parser Library
//!
//! A comprehensive Rust library for parsing and searching code elements across multiple programming languages
//! using tree-sitter. This library provides powerful tools for static code analysis, code search, and AST manipulation.
//!
//! ## Features
//!
//! - **Multi-language Support**: Parse Python, Rust, JavaScript, TypeScript, Java, C, C++, Go, and more
//! - **High Performance**: Concurrent parsing with async/await for maximum efficiency
//! - **Advanced Search**: Find functions, classes, structs, interfaces with regex pattern matching
//! - **Flexible Filtering**: Custom file filters and parsing options
//! - **Rich Metadata**: Extract detailed information about code constructs
//! - **Type Safety**: Full Rust type safety with comprehensive error handling
//! - **Configurable**: Extensive configuration options for different use cases
//!
//! ## Quick Start
//!
//! ```rust
//! use tree_parser::{parse_file, Language};
//!
//! #[tokio::main]
//! async fn main() -> Result<(), Box<dyn std::error::Error>> {
//!     // Parse a single file
//!     let parsed_file = parse_file("src/main.rs", Language::Rust).await?;
//!     
//!     println!("Found {} constructs", parsed_file.constructs.len());
//!     for construct in &parsed_file.constructs {
//!         if let Some(name) = &construct.name {
//!             println!("{}: {} (lines {}-{})", 
//!                 construct.node_type, name, 
//!                 construct.start_line, construct.end_line);
//!         }
//!     }
//!     
//!     Ok(())
//! }
//! ```
//!
//! ## Finding Code Constructs
//!
//! This library provides several powerful methods to search for specific code constructs:
//!
//! ### 1. Search by Node Type
//!
//! ```rust
//! use tree_parser::{parse_file, search_by_node_type, Language};
//!
//! #[tokio::main]
//! async fn main() -> Result<(), Box<dyn std::error::Error>> {
//!     let parsed_file = parse_file("example.py", Language::Python).await?;
//!     
//!     // Find all function definitions
//!     let functions = search_by_node_type(&parsed_file, "function_definition", None);
//!     
//!     // Find test functions using regex
//!     let test_functions = search_by_node_type(&parsed_file, "function_definition", Some(r"^test_.*"));
//!     
//!     println!("Found {} functions, {} are tests", functions.len(), test_functions.len());
//!     Ok(())
//! }
//! ```
//!
//! ### 2. Search by Multiple Node Types
//!
//! ```rust
//! use tree_parser::{parse_file, search_by_multiple_node_types, Language};
//!
//! #[tokio::main]
//! async fn main() -> Result<(), Box<dyn std::error::Error>> {
//!     let parsed_file = parse_file("example.js", Language::JavaScript).await?;
//!     
//!     // Find all function-like constructs
//!     let functions = search_by_multiple_node_types(
//!         &parsed_file,
//!         &["function_declaration", "function_expression", "arrow_function"],
//!         None
//!     );
//!     
//!     println!("Found {} function-like constructs", functions.len());
//!     Ok(())
//! }
//! ```
//!
//! ### 3. Advanced Search with Tree-sitter Queries
//!
//! ```rust
//! use tree_parser::{parse_file, search_by_query, Language};
//!
//! #[tokio::main]
//! async fn main() -> Result<(), Box<dyn std::error::Error>> {
//!     let parsed_file = parse_file("example.py", Language::Python).await?;
//!     
//!     // Find all class definitions with their methods
//!     let query = r#"
//!         (class_definition
//!           name: (identifier) @class_name
//!           body: (block
//!             (function_definition
//!               name: (identifier) @method_name)))
//!     "#;
//!     
//!     let classes_with_methods = search_by_query(&parsed_file, query)?;
//!     println!("Found {} classes with methods", classes_with_methods.len());
//!     Ok(())
//! }
//! ```
//!
//! ## Finding Node Types
//!
//! To effectively search for code constructs, you need to know the correct node types.
//! Here are the most common node types by language:
//!
//! ### Python
//! - `function_definition` - Function definitions
//! - `class_definition` - Class definitions  
//! - `import_statement` - Import statements
//! - `decorated_definition` - Functions/classes with decorators
//! - `assignment` - Variable assignments
//!
//! ### Rust
//! - `function_item` - Function definitions
//! - `struct_item` - Struct definitions
//! - `impl_item` - Implementation blocks
//! - `trait_item` - Trait definitions
//! - `enum_item` - Enum definitions
//! - `mod_item` - Module definitions
//!
//! ### JavaScript/TypeScript
//! - `function_declaration` - Function declarations
//! - `function_expression` - Function expressions
//! - `arrow_function` - Arrow functions
//! - `method_definition` - Class methods
//! - `class_declaration` - Class declarations
//!
//! ### Java
//! - `method_declaration` - Method definitions
//! - `class_declaration` - Class declarations
//! - `interface_declaration` - Interface declarations
//! - `constructor_declaration` - Constructor definitions
//!
//! For a complete list of node types, inspect your parsed files or consult the
//! tree-sitter grammar documentation for your target language.
//!
//! ### Discovering Node Types
//!
//! ```rust
//! use tree_parser::{parse_file, Language};
//! use std::collections::HashSet;
//!
//! #[tokio::main]
//! async fn main() -> Result<(), Box<dyn std::error::Error>> {
//!     let parsed_file = parse_file("your_file.py", Language::Python).await?;
//!     
//!     // Collect all unique node types
//!     let mut node_types: HashSet<String> = HashSet::new();
//!     for construct in &parsed_file.constructs {
//!         node_types.insert(construct.node_type.clone());
//!     }
//!     
//!     println!("Available node types:");
//!     for node_type in &node_types {
//!         println!("  - {}", node_type);
//!     }
//!     
//!     Ok(())
//! }
//! ```
//!
//! ### 4. Online Tree-sitter Playground
//!
//! Use the [Tree-sitter Playground](https://tree-sitter.github.io/tree-sitter/playground) to:
//! 1. Paste your code
//! 2. Select the appropriate language
//! 3. Explore the generated syntax tree
//! 4. Identify the exact node types you need
//!
//! ## Best Practices
//!
//! ### Performance Optimization
//! - Increase `max_concurrent_files` for better performance on multi-core systems
//! - Use file filters to exclude unnecessary files (node_modules, target, .git, etc.)
//! - Set appropriate `max_file_size_mb` limits to skip very large files
//! - Enable caching with `enable_caching: true` for repeated operations
//! - Use `LanguageDetection::ByExtension` for faster processing
//!
//! ### Memory Management
//! - Set `syntax_tree: None` after extracting constructs if you don't need the tree
//! - Process files in batches rather than loading entire projects
//! - Use streaming approaches for very large codebases
//!
//! ### Error Handling
//! - Always check `project.error_files` for individual file parsing errors
//! - Handle different `ErrorType` variants appropriately
//! - Use proper error propagation with `?` operator
//!
//! ## Troubleshooting
//!
//! **Common Issues:**
//! - "Unsupported language" error: Enable correct feature flags in Cargo.toml
//! - "Parse error" for valid code: Check for syntax errors or unsupported language features
//! - Poor performance: Increase concurrency, use filters, enable caching
//! - Memory issues: Drop syntax trees after use, process in batches
//! - Missing constructs: Verify node type names, check nesting, use tree-sitter queries

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

/// Main error type for the tree parser library
/// 
/// This enum represents all possible errors that can occur during parsing operations.
/// All variants are serializable and provide detailed error information.
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

/// Categorizes different types of errors for easier handling
/// 
/// This enum is used to classify errors into broad categories, making it easier
/// to implement different error handling strategies for different error types.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ErrorType {
    ParseError,
    IoError,
    UnsupportedLanguage,
    FileTooLarge,
    PermissionDenied,
}

/// Represents an error that occurred while processing a specific file
/// 
/// This struct contains detailed information about parsing failures,
/// including the file path, error type, and a descriptive message.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileError {
    pub file_path: String,
    pub error_type: ErrorType,
    pub message: String,
}

/// Supported programming languages
/// 
/// This enum represents all programming languages that the tree parser can handle.
/// Each language corresponds to a specific tree-sitter grammar.
/// 
/// # Feature Flags
/// 
/// Most languages are gated behind feature flags to reduce compilation time and binary size:
/// - `python` - Python support
/// - `rust_lang` - Rust support  
/// - `javascript` - JavaScript support
/// - `typescript` - TypeScript support
/// - `java` - Java support
/// - `c` - C support
/// - `cpp` - C++ support
/// - `go` - Go support
/// - `full` - All languages
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

/// Methods for detecting the programming language of a file
/// 
/// This enum defines different strategies for automatically determining
/// the programming language of a source code file.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum LanguageDetection {
    ByExtension,
    ByContent,
    ByShebang,
    Combined, // Uses all methods with fallback priority
}

/// Represents a function or method parameter
/// 
/// This struct contains detailed information about a parameter including
/// its name, type, default value, and whether it's variadic.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Parameter {
    pub name: String,
    pub param_type: Option<String>,
    pub default_value: Option<String>,
    pub is_variadic: bool,
}

/// Metadata associated with a code construct
/// 
/// This struct contains additional information about code constructs such as
/// visibility modifiers, parameters, return types, inheritance, and documentation.
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

/// Represents a parsed code construct (function, class, struct, etc.)
/// 
/// This is the core data structure that represents any identifiable code element
/// found during parsing. It includes the construct's location, content, metadata,
/// and hierarchical relationships with other constructs.
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

/// Represents a successfully parsed source code file
/// 
/// This struct contains all information extracted from a single file,
/// including the parsed constructs, metadata, and performance metrics.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ParsedFile {
    pub file_path: String,
    pub relative_path: String,
    pub language: Language,
    pub constructs: Vec<CodeConstruct>,
    #[serde(skip)]
    pub syntax_tree: Option<Tree>,
    pub file_size_bytes: usize,
}

/// Represents the results of parsing an entire project or directory
/// 
/// This struct aggregates the results of parsing multiple files,
/// including success metrics, error information, and language distribution.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ParsedProject {
    pub root_path: String,
    pub files: Vec<ParsedFile>,
    pub total_files_processed: usize,
    pub language_distribution: HashMap<Language, usize>,
    pub error_files: Vec<FileError>,
}

/// Filter criteria for selecting which files to parse
/// 
/// This struct allows you to specify various criteria for filtering files
/// during directory parsing operations. All criteria are optional and are
/// combined with AND logic when multiple criteria are specified.
/// 
/// # Examples
/// 
/// ```rust
/// use tree_parser::{FileFilter, Language};
/// use std::sync::Arc;
/// 
/// // Filter for Rust files only
/// let filter = FileFilter {
///     languages: Some(vec![Language::Rust]),
///     extensions: None,
///     min_size_bytes: None,
///     max_size_bytes: None,
///     custom_predicate: None,
/// };
/// 
/// // Filter with custom logic
/// let filter = FileFilter {
///     languages: None,
///     extensions: Some(vec!["rs".to_string(), "py".to_string()]),
///     min_size_bytes: Some(100),
///     max_size_bytes: Some(50_000),
///     custom_predicate: Some(Arc::new(|path| {
///         !path.to_string_lossy().contains("test")
///     })),
/// };
/// ```
#[derive(Clone)]
pub struct FileFilter {
    /// File extensions to include (e.g., ["rs", "py"]). None means all supported extensions.
    pub extensions: Option<Vec<String>>,
    /// Programming languages to include. None means all supported languages.
    pub languages: Option<Vec<Language>>,
    /// Minimum file size in bytes. Files smaller than this are excluded.
    pub min_size_bytes: Option<usize>,
    /// Maximum file size in bytes. Files larger than this are excluded.
    pub max_size_bytes: Option<usize>,
    /// Custom predicate function for advanced filtering logic
    pub custom_predicate: Option<Arc<dyn Fn(&Path) -> bool + Send + Sync>>,
}

/// Configuration options for parsing operations
/// 
/// This struct provides extensive configuration options for controlling
/// how files are parsed, including concurrency settings, file size limits,
/// and language detection strategies.
/// 
/// # Examples
/// 
/// ```rust
/// use tree_parser::{ParseOptions, LanguageDetection};
/// 
/// // Use default options
/// let options = ParseOptions::default();
/// 
/// // Custom configuration
/// let options = ParseOptions {
///     max_concurrent_files: 8,
///     include_hidden_files: false,
///     max_file_size_mb: 5,
///     recursive: true,
///     ignore_patterns: vec!["target".to_string(), "node_modules".to_string()],
///     language_detection: LanguageDetection::Combined,
///     enable_caching: true,
///     thread_pool_size: Some(4),
/// };
/// ```
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ParseOptions {
    /// Maximum number of files to parse concurrently (default: 2 * CPU cores)
    pub max_concurrent_files: usize,
    /// Whether to include hidden files (files starting with '.') in parsing
    pub include_hidden_files: bool,
    /// Maximum file size in megabytes to parse (larger files are skipped)
    pub max_file_size_mb: usize,
    /// Whether to recursively parse subdirectories
    pub recursive: bool,
    /// Patterns to ignore during directory traversal (supports glob patterns)
    pub ignore_patterns: Vec<String>,
    /// Strategy for detecting the programming language of files
    pub language_detection: LanguageDetection,
    /// Whether to enable internal caching for improved performance
    pub enable_caching: bool,
    /// Optional thread pool size (None uses system default)
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
            thread_pool_size: None, // Uses system default
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
