//! Core parsing functionality

use crate::{
    languages::*, CodeConstruct, ConstructMetadata, Error, ErrorType, FileError, Language,
    LanguageDetection, ParseOptions, ParsedFile, ParsedProject,
};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::Arc;

use compact_str::{CompactString, ToCompactString};
use memory_stats::memory_stats;
use tokio::fs;
use tokio::sync::Mutex;
use tree_sitter::{Node, Parser, Tree};
use walkdir::WalkDir;

/// Parse a single source code file and extract code constructs
/// 
/// This function reads a source code file, parses it using tree-sitter,
/// and extracts all identifiable code constructs (functions, classes, etc.).
/// 
/// # Arguments
/// 
/// * `file_path` - Path to the source code file to parse
/// * `language` - The programming language of the file
/// 
/// # Returns
/// 
/// Returns a `ParsedFile` containing all extracted constructs and metadata,
/// or an `Error` if parsing fails.
/// 
/// # Examples
/// 
/// ```rust
/// use tree_parser::{parse_file, Language};
/// 
/// #[tokio::main]
/// async fn main() -> Result<(), Box<dyn std::error::Error>> {
///     let result = parse_file("src/main.rs", Language::Rust).await?;
///     println!("Found {} constructs", result.constructs.len());
///     Ok(())
/// }
/// ```
/// 
/// # Errors
/// 
/// This function will return an error if:
/// - The file cannot be read (I/O error)
/// - The file content cannot be parsed (syntax error)
/// - The specified language is not supported
pub async fn parse_file(parser: &mut Parser, file_path: &str, language: Language, include_syntax_tree: bool) -> Result<ParsedFile, Error> {
    // Read file content
    let content: String = fs::read_to_string(file_path)
        .await
        .map_err(|e| Error::Io(e.to_string()))?;
    
    let file_size_bytes: usize = content.len();
    
    // Get tree-sitter language
    let ts_language: tree_sitter::Language = get_tree_sitter_language(&language)?;
    
    // Parse the content
    let tree: Tree = parser
        .parse(&content, None)
        .ok_or_else(|| Error::Parse("Failed to parse file".to_string()))?;
    
    // Extract code constructs
    let constructs: Vec<CodeConstruct> = extract_constructs(&tree, &content, &language);
    
    let path: &Path = Path::new(file_path);
    let relative_path: String = path
        .file_name()
        .unwrap_or_default()
        .to_string_lossy()
        .to_string();
    
    dbg!(memory_stats().unwrap());
    
    if include_syntax_tree {
        return Ok(ParsedFile {
            file_path: file_path.to_compact_string(),
            relative_path: relative_path.to_compact_string(),
            language,
            constructs,
            syntax_tree: Some(tree),
            file_size_bytes,
    
        });
    }
    
    Ok(ParsedFile {
        file_path: file_path.to_compact_string(),
        relative_path: relative_path.to_compact_string(),
        language,
        constructs,
        syntax_tree: None,
        file_size_bytes,
    })
}

/// Parse an entire project directory recursively
/// 
/// This function traverses a directory structure, identifies source code files,
/// and parses them concurrently to extract code constructs from all supported files.
/// 
/// # Arguments
/// 
/// * `dir_path` - Path to the root directory to parse
/// * `options` - Configuration options controlling parsing behavior
/// 
/// # Returns
/// 
/// Returns a `ParsedProject` containing results from all parsed files,
/// including statistics and error information.
/// 
/// # Examples
/// 
/// ```rust
/// use tree_parser::{parse_directory, ParseOptions};
/// 
/// #[tokio::main]
/// async fn main() -> Result<(), Box<dyn std::error::Error>> {
///     let options = ParseOptions::default();
///     let project = parse_directory("./src", options).await?;
///     
///     println!("Parsed {} files", project.total_files_processed);
///     for (language, count) in &project.language_distribution {
///         println!("{:?}: {} files", language, count);
///     }
///     Ok(())
/// }
/// ```
/// 
/// # Performance
/// 
/// This function uses concurrent processing to parse multiple files simultaneously.
/// The concurrency level is controlled by `options.max_concurrent_files`.
pub async fn parse_directory(
    dir_path: &str,
    options: ParseOptions,
) -> Result<ParsedProject, Error> {
    let root_path = PathBuf::from(dir_path);
    
    if !root_path.exists() {
        return Err(Error::Io(format!("Directory does not exist: {}", dir_path)));
    }
    
    // Collect files to parse
    let files_to_parse: Vec<PathBuf> = collect_files(&root_path, &options)?;
    
    // Parse files in parallel
    let (parsed_files, error_files): (Vec<ParsedFile>, Vec<FileError>) = parse_files_parallel(
        files_to_parse, &options
    ).await;
    
    // Calculate statistics
    let total_files_processed = parsed_files.len();
    let mut language_distribution = HashMap::new();
    for file in &parsed_files {
        *language_distribution.entry(file.language.clone()).or_insert(0) += 1;
    }
    
    Ok(ParsedProject {
        root_path: dir_path.to_string(),
        files: parsed_files,
        total_files_processed,
        language_distribution,
        error_files,
    })
}

/// Parse a project directory with custom file filtering
/// 
/// This function provides advanced filtering capabilities for selecting which files
/// to parse within a directory structure. It combines the standard parsing options
/// with custom filtering criteria.
/// 
/// # Arguments
/// 
/// * `dir_path` - Path to the root directory to parse
/// * `file_filter` - Custom filter criteria for file selection
/// * `options` - Configuration options controlling parsing behavior
/// 
/// # Returns
/// 
/// Returns a `ParsedProject` containing results from all files that match
/// the filter criteria.
/// 
/// # Examples
/// 
/// ```rust
/// use tree_parser::{parse_directory_with_filter, ParseOptions, FileFilter, Language};
/// use std::sync::Arc;
/// 
/// #[tokio::main]
/// async fn main() -> Result<(), Box<dyn std::error::Error>> {
///     let filter = FileFilter {
///         languages: Some(vec![Language::Rust, Language::Python]),
///         extensions: None,
///         min_size_bytes: Some(100),
///         max_size_bytes: Some(100_000),
///         custom_predicate: Some(Arc::new(|path| {
///             !path.to_string_lossy().contains("test")
///         })),
///     };
///     
///     let options = ParseOptions::default();
///     let project = parse_directory_with_filter("./src", &filter, options).await?;
///     
///     println!("Parsed {} filtered files", project.total_files_processed);
///     Ok(())
/// }
/// ```
pub async fn parse_directory_with_filter(
    dir_path: &str,
    file_filter: &crate::FileFilter,
    options: ParseOptions,
) -> Result<ParsedProject, Error> {
    let root_path = PathBuf::from(dir_path);
    
    if !root_path.exists() {
        return Err(Error::Io(format!("Directory does not exist: {}", dir_path)));
    }
    
    // Collect files to parse with custom filter
    let files_to_parse = collect_files_with_filter(&root_path, &options, file_filter)?;
    
    // Parse files in parallel
    let (parsed_files, error_files) = parse_files_parallel(files_to_parse, &options).await;
    
    // Calculate statistics
    let total_files_processed = parsed_files.len();
    let mut language_distribution = HashMap::new();
    for file in &parsed_files {
        *language_distribution.entry(file.language.clone()).or_insert(0) += 1;
    }
    
    Ok(ParsedProject {
        root_path: dir_path.to_string(),
        files: parsed_files,
        total_files_processed,

        language_distribution,
        error_files,
    })
}

/// Collect files to parse from directory based on parsing options
/// 
/// This internal function traverses a directory structure and collects all files
/// that should be parsed according to the provided options.
/// 
/// # Arguments
/// 
/// * `root_path` - Root directory path to start collection from
/// * `options` - Parsing options that control file selection
/// 
/// # Returns
/// 
/// A vector of file paths that should be parsed, or an error if directory
/// traversal fails.
fn collect_files(root_path: &Path, options: &ParseOptions) -> Result<Vec<PathBuf>, Error> {
    let mut files = Vec::new();
    
    let walker = if options.recursive {
        WalkDir::new(root_path)
    } else {
        WalkDir::new(root_path).max_depth(1)
    };
    
    for entry in walker {
        let entry = entry.map_err(|e| Error::Io(e.to_string()))?;
        let path = entry.path();
        
        // Skip directories
        if path.is_dir() {
            continue;
        }
        
        // Skip hidden files if not included
        if !options.include_hidden_files && is_hidden_file(path) {
            continue;
        }
        
        // Check ignore patterns
        if should_ignore_file(path, &options.ignore_patterns) {
            continue;
        }
        
        // Check file size
        if let Ok(metadata) = path.metadata() {
            let size_mb = metadata.len() as usize / (1024 * 1024);
            if size_mb > options.max_file_size_mb {
                continue;
            }
        }
        
        // Check if we can detect the language
        if detect_language_by_extension(&path.to_string_lossy()).is_some() {
            files.push(path.to_path_buf());
        }
    }
    
    Ok(files)
}

/// Collect files with custom filter criteria
/// 
/// This internal function extends the basic file collection with additional
/// filtering capabilities provided by a `FileFilter`.
/// 
/// # Arguments
/// 
/// * `root_path` - Root directory path to start collection from
/// * `options` - Parsing options that control file selection
/// * `filter` - Custom filter criteria for more precise file selection
/// 
/// # Returns
/// 
/// A vector of file paths that match both the parsing options and the custom
/// filter criteria.
fn collect_files_with_filter(
    root_path: &Path,
    options: &ParseOptions,
    filter: &crate::FileFilter,
) -> Result<Vec<PathBuf>, Error> {
    let mut files = collect_files(root_path, options)?;
    
    // Apply custom filter
    files.retain(|path| {
        // Check extensions
        if let Some(ref extensions) = filter.extensions {
            if let Some(ext) = path.extension() {
                if !extensions.contains(&ext.to_string_lossy().to_lowercase()) {
                    return false;
                }
            } else {
                return false;
            }
        }
        
        // Check languages
        if let Some(ref languages) = filter.languages {
            if let Some(detected_lang) = detect_language_by_extension(&path.to_string_lossy()) {
                if !languages.contains(&detected_lang) {
                    return false;
                }
            } else {
                return false;
            }
        }
        
        // Check file size
        if let Ok(metadata) = path.metadata() {
            let size = metadata.len() as usize;
            
            if let Some(min_size) = filter.min_size_bytes {
                if size < min_size {
                    return false;
                }
            }
            
            if let Some(max_size) = filter.max_size_bytes {
                if size > max_size {
                    return false;
                }
            }
        }
        
        // Apply custom predicate
        if let Some(ref predicate) = filter.custom_predicate {
            if !predicate(path) {
                return false;
            }
        }
        
        true
    });
    
    Ok(files)
}

/// Parse files in parallel
async fn parse_files_parallel(
    files: Vec<PathBuf>,
    options: &ParseOptions,
) -> (Vec<ParsedFile>, Vec<FileError>) {
    let chunk_size: usize = std::cmp::max(1, files.len() / options.max_concurrent_files);
    let mut parsed_files: Vec<ParsedFile> = Vec::new();
    let mut error_files: Vec<FileError> = Vec::new();
    
    let parsers: Arc<Mutex<Vec<Parser>>> = Arc::new(Mutex::new(Vec::new()));
    
    for chunk in files.chunks(chunk_size) {
        let mut chunk_results = Vec::new();
        
        for subchunk in chunk {
            let parsers = parsers.clone();
            chunk_results.push(async move {
                let path_str: String = subchunk.as_path().to_string_lossy().to_string();
                
                // Detect language
                let language: Option<Language> = match options.language_detection {
                    LanguageDetection::ByExtension => detect_language_by_extension(&path_str),
                    LanguageDetection::Combined => {
                        // Try to read content for better detection
                        if let Ok(content) = tokio::fs::read_to_string(subchunk.as_path()).await {
                            detect_language(&path_str, Some(&content))
                        } else {
                            detect_language_by_extension(&path_str)
                        }
                    }
                    _ => detect_language_by_extension(&path_str), // Fallback
                };
                
                if let Some(lang) = language {
                    let mut parsers: tokio::sync::MutexGuard<'_, Vec<Parser>> = parsers.lock().await;
                    let mut parser = match get_parser(&mut parsers, &lang) {
                        Ok(parser) => parser,
                        Err(_) => return Err(FileError {
                            file_path: path_str,
                            error_type: ErrorType::ParseError,
                            message: "Failed to get a suitable parser".to_string(),
                        }),
                    };
                    
                    match parse_file(&mut parser, &path_str, lang, options.include_syntax_tree).await {
                        Ok(parsed) => Ok(parsed),
                        Err(e) => Err(FileError {
                            file_path: path_str,
                            error_type: ErrorType::ParseError,
                            message: e.to_string(),
                        }),
                    }
                } else {
                    Err(FileError {
                        file_path: path_str,
                        error_type: ErrorType::UnsupportedLanguage,
                        message: "Could not detect language".to_string(),
                    })
                }
            })
        }
        
        // Await all tasks in this chunk
        for result in futures::future::join_all(chunk_results).await {
            match result {
                Ok(parsed_file) => parsed_files.push(parsed_file),
                Err(error) => error_files.push(error),
            }
        }
    }
    
    (parsed_files, error_files)
}

fn get_parser<'a, 'b>(parsers: &'a mut Vec<Parser>, lang: &'b Language) -> Result<&'a mut Parser, Box<dyn std::error::Error>> {
    let ts_language: tree_sitter::Language = get_tree_sitter_language(lang)?;
    
    // Pre-initialize necessary parsers
    for (index, parser) in parsers.iter().enumerate() {
        // Only when we do not have a parser for the detected language,
        // we initialize a new parser
        if let Some(parser_language) = parser.language() {
            if ts_language.name() == parser_language.name() {
                return Ok(&mut parsers[index]);
            }
        } 
    }
    
    // Create parser
    let mut parser: Parser = Parser::new();
    parser.set_language(&ts_language)?;
    parsers.push(parser);
    
    Ok(parsers.last_mut().unwrap())
}

/// Extract code constructs from syntax tree
fn extract_constructs(tree: &Tree, source: &str, language: &Language) -> Vec<CodeConstruct> {
    let root_node = tree.root_node();
    let mut root_constructs = Vec::new();
    
    // Extract constructs with proper parent-child relationships
    extract_constructs_hierarchical(root_node, source, language, &mut root_constructs, None);
    
    // Flatten the hierarchy for the final result while preserving relationships
    let mut all_constructs = Vec::new();
    flatten_constructs(&root_constructs, &mut all_constructs);
    
    all_constructs
}

/// Recursively extract constructs from nodes with proper hierarchy
fn extract_constructs_hierarchical(
    node: Node,
    source: &str,
    language: &Language,
    constructs: &mut Vec<CodeConstruct>,
    parent_construct: Option<&CodeConstruct>,
) {
    let node_type = node.kind();
    let supported_types = get_supported_node_types(language);
    
    if supported_types.contains(&node_type.to_string()) {
        let mut construct = create_code_construct_with_parent(node, source, language, parent_construct);
        
        // Recursively process children and add them to this construct
        let mut child_constructs = Vec::new();
        for i in 0..node.child_count() {
            if let Some(child) = node.child(i) {
                extract_constructs_hierarchical(child, source, language, &mut child_constructs, Some(&construct));
            }
        }
        
        construct.children = child_constructs;
        constructs.push(construct);
    } else {
        // If this node is not a supported construct, continue searching in its children
        for i in 0..node.child_count() {
            if let Some(child) = node.child(i) {
                extract_constructs_hierarchical(child, source, language, constructs, parent_construct);
            }
        }
    }
}

/// Flatten hierarchical constructs into a single vector while preserving relationships
fn flatten_constructs(constructs: &[CodeConstruct], flattened: &mut Vec<CodeConstruct>) {
    for construct in constructs {
        flattened.push(construct.clone());
        flatten_constructs(&construct.children, flattened);
    }
}

/// Create a CodeConstruct from a tree-sitter node with proper parent relationship
fn create_code_construct_with_parent(
    node: Node, 
    source: &str, 
    language: &Language,
    parent_construct: Option<&CodeConstruct>
) -> CodeConstruct {
    let start_byte = node.start_byte();
    let end_byte = node.end_byte();
    let source_code = source[start_byte..end_byte].to_string();
    
    let start_point = node.start_position();
    let end_point = node.end_position();
    
    // Extract name if possible
    let name: Option<CompactString> = extract_construct_name(node, source);
    
    // Create metadata
    let metadata: ConstructMetadata = extract_metadata(node, source, language);
    
    // Set parent if provided
    let parent: Option<Box<CodeConstruct>> = parent_construct.map(|p| Box::new(p.clone()));
    
    CodeConstruct {
        node_type: node.kind().to_compact_string(),
        name,
        source_code: source_code.to_compact_string(),
        start_line: start_point.row + 1, // Convert to 1-based
        end_line: end_point.row + 1,
        start_byte,
        end_byte,
        parent,
        children: Vec::new(), // Will be populated by the caller
        metadata,
    }
}

/// Extract construct name from node
fn extract_construct_name(node: Node, source: &str) -> Option<CompactString> {
    // Try to find identifier child
    for i in 0..node.child_count() {
        if let Some(child) = node.child(i) {
            if child.kind() == "identifier" || child.kind() == "name" {
                let start = child.start_byte();
                let end = child.end_byte();
                return Some(source[start..end].to_string().to_compact_string());
            }
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Language;

    #[test]
    fn test_parent_child_relationships() {
        // Simple Python code with nested structure
        let source = "class TestClass:\n    def test_method(self):\n        pass";
        
        // Create a simple tree-sitter parser for testing
        let mut parser = Parser::new();
        let language = crate::languages::get_tree_sitter_language(&Language::Python).unwrap();
        parser.set_language(&language).unwrap();
        
        let tree = parser.parse(source, None).unwrap();
        let constructs = extract_constructs(&tree, source, &Language::Python);
        
        // Find class and method constructs
        let class_construct = constructs.iter().find(|c| c.node_type == "class_definition");
        let method_construct = constructs.iter().find(|c| c.node_type == "function_definition");
        
        assert!(class_construct.is_some(), "Should find class construct");
        assert!(method_construct.is_some(), "Should find method construct");
        
        let method = method_construct.unwrap();
        
        // Check that method has a parent
        assert!(method.parent.is_some(), "Method should have a parent");
        
        if let Some(parent) = &method.parent {
            assert_eq!(parent.node_type, "class_definition", "Method's parent should be the class");
        }
        
        // Check that class has children
        let class = class_construct.unwrap();
        assert!(!class.children.is_empty(), "Class should have children");
        
        let child_method = class.children.iter().find(|c| c.node_type == "function_definition");
        assert!(child_method.is_some(), "Class should contain the method as a child");
    }
}

/// Extract metadata from node
fn extract_metadata(_node: Node, _source: &str, _language: &Language) -> ConstructMetadata {
    ConstructMetadata {
        visibility: None,
        modifiers: Vec::new(),
        parameters: Vec::new(),
        return_type: None,
        inheritance: Vec::new(),
        annotations: Vec::new(),
        documentation: None,
    }
}

/// Check if file is hidden
fn is_hidden_file(path: &Path) -> bool {
    path.file_name()
        .and_then(|name| name.to_str())
        .map(|name| name.starts_with('.'))
        .unwrap_or(false)
}

/// Check if file should be ignored based on patterns
fn should_ignore_file(path: &Path, ignore_patterns: &[String]) -> bool {
    let path_str = path.to_string_lossy();
    
    for pattern in ignore_patterns {
        if path_str.contains(pattern) {
            return true;
        }
    }
    
    false
}