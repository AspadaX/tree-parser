# Goal

# Tree Parser Library

This library provides a comprehensive solution for parsing and searching code elements across multiple programming languages. It supports finding functions, classes, structs, interfaces, and other code constructs with flexible search capabilities.

The library will return the codes of the specified functions, classes, structs, interfaces and other code constructs. For example, in Python, if we search a function called `hello`, then it will return the full definition of `hello` and relevant information like which class and modules it belongs to.

It uses tree-sitter as a backend, and supports all languages supported by tree-sitter.

# Tech Stacks

Langauge: Rust 2024

# APIs defined

## Core Functions

### `parse_file(file_path: &str, language: Language) -> Result<ParsedFile, Error>`
Parses a single source code file and returns a structured representation of its contents.

**Parameters:**
- `file_path`: Path to the source code file
- `language`: Programming language enum variant (e.g., `Language::Python`, `Language::Rust`, `Language::JavaScript`)

**Returns:**
- `ParsedFile`: Structured representation containing all code elements
- `Error`: Parsing error if the file cannot be processed

### `parse_directory(dir_path: &str, options: ParseOptions) -> Result<ParsedProject, Error>`
Parses an entire project directory concurrently, automatically detecting file types and processing them in parallel for maximum efficiency.

**Parameters:**
- `dir_path`: Path to the project directory
- `options`: Configuration options for parsing behavior

**Returns:**
- `ParsedProject`: Complete project structure with all parsed files
- `Error`: Parsing error if the directory cannot be processed

### `parse_directory_with_filter(dir_path: &str, file_filter: FileFilter, options: ParseOptions) -> Result<ParsedProject, Error>`
Parses a project directory with custom file filtering, using Rust's async/await and rayon for concurrent processing.

**Parameters:**
- `dir_path`: Path to the project directory
- `file_filter`: Custom filter to select which files to parse
- `options`: Configuration options for parsing behavior

**Returns:**
- `ParsedProject`: Filtered project structure with parsed files
- `Error`: Parsing error if the directory cannot be processed

### `search_by_node_type(parsed_file: &ParsedFile, node_type: &str, name_pattern: Option<&str>) -> Vec<CodeConstruct>`
Searches for code constructs by tree-sitter node type with optional name filtering.

**Parameters:**
- `parsed_file`: Previously parsed file structure
- `node_type`: Tree-sitter node type (e.g., "function_definition", "class_definition", "struct_item")
- `name_pattern`: Optional regex pattern to filter by construct name

**Returns:**
- `Vec<CodeConstruct>`: List of matching code constructs with their full definitions

### `search_by_multiple_node_types(parsed_file: &ParsedFile, node_types: &[&str], name_pattern: Option<&str>) -> Vec<CodeConstruct>`
Searches for code constructs matching any of the specified tree-sitter node types.

**Parameters:**
- `parsed_file`: Previously parsed file structure
- `node_types`: Array of tree-sitter node types to search for
- `name_pattern`: Optional regex pattern to filter by construct name

**Returns:**
- `Vec<CodeConstruct>`: List of matching code constructs with their full definitions

### `get_supported_node_types(language: Language) -> Vec<String>`
Returns all supported tree-sitter node types for a given language.

**Parameters:**
- `language`: Programming language enum variant

**Returns:**
- `Vec<String>`: List of all available node types for the language

### `search_by_query(parsed_file: &ParsedFile, tree_sitter_query: &str) -> Vec<CodeConstruct>`
Executes a custom tree-sitter query for maximum flexibility.

**Parameters:**
- `parsed_file`: Previously parsed file structure
- `tree_sitter_query`: Raw tree-sitter query string

**Returns:**
- `Vec<CodeConstruct>`: List of matching code constructs

## Data Structures

### `Language`
```rust
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
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
```

### `ParsedProject`
```rust
pub struct ParsedProject {
    pub root_path: String,
    pub files: Vec<ParsedFile>,
    pub total_files_processed: usize,
    pub processing_time_ms: u64,
    pub language_distribution: HashMap<Language, usize>,
    pub error_files: Vec<FileError>,
}
```

### `ParsedFile`
```rust
pub struct ParsedFile {
    pub file_path: String,
    pub relative_path: String,
    pub language: Language,
    pub constructs: Vec<CodeConstruct>,
    pub syntax_tree: tree_sitter::Tree,
    pub file_size_bytes: usize,
    pub parse_time_ms: u64,
}
```

### `ParseOptions`
```rust
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
```

### `FileFilter`
```rust
pub struct FileFilter {
    pub extensions: Option<Vec<String>>,
    pub languages: Option<Vec<Language>>,
    pub min_size_bytes: Option<usize>,
    pub max_size_bytes: Option<usize>,
    pub custom_predicate: Option<Box<dyn Fn(&Path) -> bool + Send + Sync>>,
}
```

### `LanguageDetection`
```rust
#[derive(Debug, Clone)]
pub enum LanguageDetection {
    ByExtension,
    ByContent,
    ByShebang,
    Combined, // Uses all methods with fallback priority
}
```

### `FileError`
```rust
pub struct FileError {
    pub file_path: String,
    pub error_type: ErrorType,
    pub message: String,
}

#[derive(Debug, Clone)]
pub enum ErrorType {
    ParseError,
    IoError,
    UnsupportedLanguage,
    FileTooLarge,
    PermissionDenied,
}
```

### `CodeConstruct`
```rust
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
```

### `ConstructMetadata`
```rust
pub struct ConstructMetadata {
    pub visibility: Option<String>,
    pub modifiers: Vec<String>,
    pub parameters: Vec<Parameter>,
    pub return_type: Option<String>,
    pub inheritance: Vec<String>,
    pub annotations: Vec<String>,
    pub documentation: Option<String>,
}
```

### `Parameter`
```rust
pub struct Parameter {
    pub name: String,
    pub param_type: Option<String>,
    pub default_value: Option<String>,
    pub is_variadic: bool,
}
```
