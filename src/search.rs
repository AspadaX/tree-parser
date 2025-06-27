//! Search functionality for finding code constructs

use crate::{languages::get_tree_sitter_language, CodeConstruct, Error, Language, ParsedFile};
use regex::Regex;
use tree_sitter::{Query, QueryCursor};
use streaming_iterator::StreamingIterator;

/// Search for code constructs by their tree-sitter node type
/// 
/// This function searches through all code constructs in a parsed file
/// and returns those that match the specified node type. Optionally,
/// results can be filtered by a regex pattern applied to construct names.
/// 
/// # Arguments
/// 
/// * `parsed_file` - The parsed file to search within
/// * `node_type` - The tree-sitter node type to search for (e.g., "function_definition")
/// * `name_pattern` - Optional regex pattern to filter results by construct name
/// 
/// # Returns
/// 
/// A vector of `CodeConstruct` objects that match the search criteria.
/// 
/// # Examples
/// 
/// ```rust
/// use tree_parser::{parse_file, search_by_node_type, Language};
/// 
/// #[tokio::main]
/// async fn main() -> Result<(), Box<dyn std::error::Error>> {
///     let parsed = parse_file("example.py", Language::Python).await?;
///     
///     // Find all function definitions
///     let functions = search_by_node_type(&parsed, "function_definition", None);
///     
///     // Find functions with names starting with "test_"
///     let test_functions = search_by_node_type(&parsed, "function_definition", Some(r"^test_"));
///     
///     println!("Found {} functions, {} are tests", functions.len(), test_functions.len());
///     Ok(())
/// }
/// ```
pub fn search_by_node_type(
    parsed_file: &ParsedFile,
    node_type: &str,
    name_pattern: Option<&str>,
) -> Vec<CodeConstruct> {
    let mut results = Vec::new();
    
    // Compile regex pattern if provided
    let regex = if let Some(pattern) = name_pattern {
        match Regex::new(pattern) {
            Ok(r) => Some(r),
            Err(_) => return results, // Invalid regex, return empty results
        }
    } else {
        None
    };
    
    // Search through all constructs
    for construct in &parsed_file.constructs {
        if construct.node_type == node_type {
            // Check name pattern if provided
            if let Some(ref regex) = regex {
                if let Some(ref name) = construct.name {
                    if regex.is_match(name) {
                        results.push(construct.clone());
                    }
                }
            } else {
                results.push(construct.clone());
            }
        }
        
        // Search recursively in children
        search_in_children(construct, node_type, &regex, &mut results);
    }
    
    results
}

/// Search for code constructs matching any of the specified node types
/// 
/// This function extends `search_by_node_type` to search for multiple node types
/// simultaneously. This is useful when looking for related constructs that may
/// have different node types in different languages.
/// 
/// # Arguments
/// 
/// * `parsed_file` - The parsed file to search within
/// * `node_types` - Array of tree-sitter node types to search for
/// * `name_pattern` - Optional regex pattern to filter results by construct name
/// 
/// # Returns
/// 
/// A vector of `CodeConstruct` objects that match any of the specified node types
/// and the optional name pattern.
/// 
/// # Examples
/// 
/// ```rust
/// use tree_parser::{parse_file, search_by_multiple_node_types, Language};
/// 
/// #[tokio::main]
/// async fn main() -> Result<(), Box<dyn std::error::Error>> {
///     let parsed = parse_file("example.js", Language::JavaScript).await?;
///     
///     // Find all function-like constructs
///     let functions = search_by_multiple_node_types(
///         &parsed,
///         &["function_declaration", "function_expression", "arrow_function"],
///         None
///     );
///     
///     println!("Found {} function-like constructs", functions.len());
///     Ok(())
/// }
/// ```
pub fn search_by_multiple_node_types(
    parsed_file: &ParsedFile,
    node_types: &[&str],
    name_pattern: Option<&str>,
) -> Vec<CodeConstruct> {
    let mut results = Vec::new();
    
    // Compile regex pattern if provided
    let regex = if let Some(pattern) = name_pattern {
        match Regex::new(pattern) {
            Ok(r) => Some(r),
            Err(_) => return results, // Invalid regex, return empty results
        }
    } else {
        None
    };
    
    // Search through all constructs
    for construct in &parsed_file.constructs {
        if node_types.contains(&construct.node_type.as_str()) {
            // Check name pattern if provided
            if let Some(ref regex) = regex {
                if let Some(ref name) = construct.name {
                    if regex.is_match(name) {
                        results.push(construct.clone());
                    }
                }
            } else {
                results.push(construct.clone());
            }
        }
        
        // Search recursively in children
        search_in_children_multiple(construct, node_types, &regex, &mut results);
    }
    
    results
}

/// Execute a custom tree-sitter query for advanced searching
/// 
/// This function allows you to use tree-sitter's powerful query language
/// to perform complex searches on the syntax tree. This provides the most
/// flexibility for finding specific code patterns.
/// 
/// # Arguments
/// 
/// * `parsed_file` - The parsed file to search within
/// * `tree_sitter_query` - A tree-sitter query string
/// 
/// # Returns
/// 
/// A `Result` containing a vector of `CodeConstruct` objects that match
/// the query, or an `Error` if the query is invalid or execution fails.
/// 
/// # Examples
/// 
/// ```rust
/// use tree_parser::{parse_file, search_by_query, Language};
/// 
/// #[tokio::main]
/// async fn main() -> Result<(), Box<dyn std::error::Error>> {
///     let parsed = parse_file("example.py", Language::Python).await?;
///     
///     // Find all function definitions with decorators
///     let query = r#"
///         (decorated_definition
///           (function_definition
///             name: (identifier) @func_name))
///     "#;
///     
///     let decorated_functions = search_by_query(&parsed, query)?;
///     println!("Found {} decorated functions", decorated_functions.len());
///     Ok(())
/// }
/// ```
/// 
/// # Errors
/// 
/// This function will return an error if:
/// - The query syntax is invalid
/// - The syntax tree is not available
/// - File I/O operations fail
pub fn search_by_query(
    parsed_file: &ParsedFile,
    tree_sitter_query: &str,
) -> Result<Vec<CodeConstruct>, Error> {
    let mut results = Vec::new();
    
    // Get the syntax tree
    let tree = parsed_file.syntax_tree.as_ref()
        .ok_or_else(|| Error::Parse("No syntax tree available".to_string()))?;
    
    // Get the tree-sitter language
    let ts_language = get_tree_sitter_language(&parsed_file.language)?;
    
    // Create and execute query
    let query = Query::new(&ts_language, tree_sitter_query)
        .map_err(|e| Error::InvalidQuery(e.to_string()))?;
    
    let mut cursor = QueryCursor::new();
    
    // Read the source code to extract text
    let source = std::fs::read_to_string(&parsed_file.file_path)
        .map_err(|e| Error::Io(e.to_string()))?;
    
    let mut matches = cursor.matches(&query, tree.root_node(), source.as_bytes());
    while let Some(query_match) = matches.next() {
        for capture in query_match.captures {
            let node = capture.node;
            let construct = create_code_construct_from_node(node, &source, &parsed_file.language);
            results.push(construct);
        }
    }
    
    Ok(results)
}

/// Search for function definitions in a parsed file
/// 
/// This is a convenience function that searches for function-like constructs
/// across different programming languages. It automatically selects the
/// appropriate node types based on the file's language.
/// 
/// # Arguments
/// 
/// * `parsed_file` - The parsed file to search within
/// * `name_pattern` - Optional regex pattern to filter results by function name
/// 
/// # Returns
/// 
/// A vector of `CodeConstruct` objects representing functions, methods,
/// or other callable constructs.
/// 
/// # Supported Languages
/// 
/// - **Python**: `function_definition`
/// - **Rust**: `function_item`
/// - **JavaScript/TypeScript**: `function_declaration`, `function_expression`, `arrow_function`
/// - **Java**: `method_declaration`, `constructor_declaration`
/// - **C/C++**: `function_definition`
/// - **Go**: `function_declaration`, `method_declaration`
/// 
/// # Examples
/// 
/// ```rust
/// use tree_parser::{parse_file, search_functions, Language};
/// 
/// #[tokio::main]
/// async fn main() -> Result<(), Box<dyn std::error::Error>> {
///     let parsed = parse_file("example.rs", Language::Rust).await?;
///     
///     // Find all functions
///     let all_functions = search_functions(&parsed, None);
///     
///     // Find functions starting with "test_"
///     let test_functions = search_functions(&parsed, Some(r"^test_"));
///     
///     println!("Found {} functions, {} are tests", all_functions.len(), test_functions.len());
///     Ok(())
/// }
/// ```
pub fn search_functions(
    parsed_file: &ParsedFile,
    name_pattern: Option<&str>,
) -> Vec<CodeConstruct> {
    let function_types = match parsed_file.language {
        Language::Python => vec!["function_definition"],
        Language::Rust => vec!["function_item"],
        Language::JavaScript => vec!["function_declaration", "function_expression", "arrow_function"],
        Language::TypeScript => vec!["function_declaration", "function_expression", "arrow_function"],
        Language::Java => vec!["method_declaration", "constructor_declaration"],
        Language::C => vec!["function_definition"],
        Language::Cpp => vec!["function_definition"],
        Language::Go => vec!["function_declaration", "method_declaration"],
        _ => vec![],
    };
    
    search_by_multiple_node_types(parsed_file, &function_types, name_pattern)
}

/// Search for class and type definitions in a parsed file
/// 
/// This is a convenience function that searches for class-like constructs
/// across different programming languages. It automatically selects the
/// appropriate node types based on the file's language.
/// 
/// # Arguments
/// 
/// * `parsed_file` - The parsed file to search within
/// * `name_pattern` - Optional regex pattern to filter results by class/type name
/// 
/// # Returns
/// 
/// A vector of `CodeConstruct` objects representing classes, structs,
/// interfaces, enums, or other type definitions.
/// 
/// # Supported Languages
/// 
/// - **Python**: `class_definition`
/// - **Rust**: `struct_item`, `enum_item`
/// - **JavaScript**: `class_declaration`
/// - **TypeScript**: `class_declaration`, `interface_declaration`
/// - **Java**: `class_declaration`, `interface_declaration`
/// - **C**: `struct_specifier`, `union_specifier`, `enum_specifier`
/// - **C++**: `class_specifier`, `struct_specifier`, `union_specifier`, `enum_specifier`
/// - **Go**: `type_declaration`
/// 
/// # Examples
/// 
/// ```rust
/// use tree_parser::{parse_file, search_classes, Language};
/// 
/// #[tokio::main]
/// async fn main() -> Result<(), Box<dyn std::error::Error>> {
///     let parsed = parse_file("example.py", Language::Python).await?;
///     
///     // Find all class definitions
///     let all_classes = search_classes(&parsed, None);
///     
///     // Find classes with names ending in "Error"
///     let error_classes = search_classes(&parsed, Some(r"Error$"));
///     
///     println!("Found {} classes, {} are error types", all_classes.len(), error_classes.len());
///     Ok(())
/// }
/// ```
pub fn search_classes(
    parsed_file: &ParsedFile,
    name_pattern: Option<&str>,
) -> Vec<CodeConstruct> {
    let class_types = match parsed_file.language {
        Language::Python => vec!["class_definition"],
        Language::Rust => vec!["struct_item", "enum_item"],
        Language::JavaScript => vec!["class_declaration"],
        Language::TypeScript => vec!["class_declaration", "interface_declaration"],
        Language::Java => vec!["class_declaration", "interface_declaration"],
        Language::C => vec!["struct_specifier", "union_specifier", "enum_specifier"],
        Language::Cpp => vec!["class_specifier", "struct_specifier", "union_specifier", "enum_specifier"],
        Language::Go => vec!["type_declaration"],
        _ => vec![],
    };
    
    search_by_multiple_node_types(parsed_file, &class_types, name_pattern)
}

/// Search for imports in a parsed file
pub fn search_imports(parsed_file: &ParsedFile) -> Vec<CodeConstruct> {
    let import_types = match parsed_file.language {
        Language::Python => vec!["import_statement", "import_from_statement"],
        Language::Rust => vec!["use_declaration"],
        Language::JavaScript => vec!["import_statement"],
        Language::TypeScript => vec!["import_statement"],
        Language::Java => vec!["import_declaration"],
        Language::C => vec!["preproc_include"],
        Language::Cpp => vec!["preproc_include"],
        Language::Go => vec!["import_declaration"],
        _ => vec![],
    };
    
    search_by_multiple_node_types(parsed_file, &import_types, None)
}

/// Search for variables/constants in a parsed file
pub fn search_variables(
    parsed_file: &ParsedFile,
    name_pattern: Option<&str>,
) -> Vec<CodeConstruct> {
    let variable_types = match parsed_file.language {
        Language::Python => vec!["assignment"],
        Language::Rust => vec!["const_item", "static_item"],
        Language::JavaScript => vec!["variable_declaration"],
        Language::TypeScript => vec!["variable_declaration"],
        Language::Java => vec!["field_declaration"],
        Language::C => vec!["declaration"],
        Language::Cpp => vec!["declaration"],
        Language::Go => vec!["var_declaration", "const_declaration"],
        _ => vec![],
    };
    
    search_by_multiple_node_types(parsed_file, &variable_types, name_pattern)
}

/// Helper function to search recursively in children
fn search_in_children(
    construct: &CodeConstruct,
    node_type: &str,
    regex: &Option<Regex>,
    results: &mut Vec<CodeConstruct>,
) {
    for child in &construct.children {
        if child.node_type == node_type {
            // Check name pattern if provided
            if let Some(regex) = regex {
                if let Some(name) = &child.name {
                    if regex.is_match(name) {
                        results.push(child.clone());
                    }
                }
            } else {
                results.push(child.clone());
            }
        }
        
        // Continue searching recursively
        search_in_children(child, node_type, regex, results);
    }
}

/// Helper function to search recursively in children for multiple node types
fn search_in_children_multiple(
    construct: &CodeConstruct,
    node_types: &[&str],
    regex: &Option<Regex>,
    results: &mut Vec<CodeConstruct>,
) {
    for child in &construct.children {
        if node_types.contains(&child.node_type.as_str()) {
            // Check name pattern if provided
            if let Some(regex) = regex {
                if let Some(name) = &child.name {
                    if regex.is_match(name) {
                        results.push(child.clone());
                    }
                }
            } else {
                results.push(child.clone());
            }
        }
        
        // Continue searching recursively
        search_in_children_multiple(child, node_types, regex, results);
    }
}

/// Create a CodeConstruct from a tree-sitter node (used in query search)
fn create_code_construct_from_node(
    node: tree_sitter::Node,
    source: &str,
    _language: &Language,
) -> CodeConstruct {
    let start_byte = node.start_byte();
    let end_byte = node.end_byte();
    let source_code = source[start_byte..end_byte].to_string();
    
    let start_point = node.start_position();
    let end_point = node.end_position();
    
    // Extract name if possible
    let name = extract_node_name(node, source);
    
    CodeConstruct {
        node_type: node.kind().to_string(),
        name,
        source_code,
        start_line: start_point.row + 1, // Convert to 1-based
        end_line: end_point.row + 1,
        start_byte,
        end_byte,
        parent: None,
        children: Vec::new(),
        metadata: crate::ConstructMetadata {
            visibility: None,
            modifiers: Vec::new(),
            parameters: Vec::new(),
            return_type: None,
            inheritance: Vec::new(),
            annotations: Vec::new(),
            documentation: None,
        },
    }
}

/// Extract name from a tree-sitter node
fn extract_node_name(node: tree_sitter::Node, source: &str) -> Option<String> {
    // Try to find identifier child
    for i in 0..node.child_count() {
        if let Some(child) = node.child(i) {
            if child.kind() == "identifier" || child.kind() == "name" {
                let start = child.start_byte();
                let end = child.end_byte();
                return Some(source[start..end].to_string());
            }
        }
    }
    None
}