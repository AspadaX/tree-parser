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
    
    // Search through all constructs (already flattened, no need for recursive search)
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
    
    // Search through all constructs (already flattened, no need for recursive search)
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{parse_file, Language};
    use std::fs;
    use tokio;

    #[tokio::test]
    async fn test_no_duplicate_results() {
        // Create a test Python file with nested functions
        let test_content = r#"
class CacheEngine:
    def __init__(self):
        pass
    
    def _allocate_kv_cache(self):
        return "cache allocated"

    class InnerClass:
        def _allocate_kv_cache(self):
            return "inner cache"
"#;
        
        // Write test file
        let test_file = "test_cache_duplication.py";
        fs::write(test_file, test_content).expect("Failed to write test file");
        
        // Parse the file
        let parsed = parse_file(test_file, Language::Python).await.expect("Failed to parse file");
        
        // Search for function definitions with specific name
        let functions = search_by_node_type(&parsed, "function_definition", Some("_allocate_kv_cache"));
        
        // Should find exactly 2 functions (one in CacheEngine, one in InnerClass)
        // Before the fix, this would return 4 (each function counted twice due to duplication)
        assert_eq!(functions.len(), 2, "Expected exactly 2 functions, but found {}", functions.len());
        
        // Verify the functions have different parents
        let mut parent_names = Vec::new();
        for func in &functions {
            if let Some(parent) = &func.parent {
                if let Some(parent_name) = &parent.name {
                    parent_names.push(parent_name.clone());
                }
            }
        }
        
        // Should have 2 different parent classes
        parent_names.sort();
        parent_names.dedup();
        assert_eq!(parent_names.len(), 2, "Expected 2 different parent classes");
        
        // Clean up
        fs::remove_file(test_file).ok();
    }
}