# Tree Parser

[![Crates.io](https://img.shields.io/crates/v/tree-parser.svg)](https://crates.io/crates/tree-parser)
[![Documentation](https://docs.rs/tree-parser/badge.svg)](https://docs.rs/tree-parser)
[![License](https://img.shields.io/badge/license-MIT-blue.svg)](LICENSE)

A comprehensive Rust library for parsing and searching code elements across multiple programming languages using tree-sitter. This library provides powerful tools for static code analysis, code search, and AST manipulation.

## Features

- 🚀 **Multi-language Support**: Parse Python, Rust, JavaScript, TypeScript, Java, C, C++, Go, and more
- ⚡ **High Performance**: Concurrent parsing with async/await for maximum efficiency
- 🔍 **Advanced Search**: Find functions, classes, structs, interfaces with regex pattern matching
- 🎯 **Flexible Filtering**: Custom file filters and parsing options
- 📊 **Rich Metadata**: Extract detailed information about code constructs
- 🛡️ **Type Safety**: Full Rust type safety with comprehensive error handling
- 🔧 **Configurable**: Extensive configuration options for different use cases

## Quick Start

Add this to your `Cargo.toml`:

```toml
[dependencies]
tree-parser = "0.1.0"

# Enable specific language features
tree-parser = { version = "0.1.0", features = ["python", "rust_lang", "javascript"] }

# Or enable all languages
tree-parser = { version = "0.1.0", features = ["full"] }
```

## Basic Usage

### Parse a Single File

```rust
use tree_parser::{parse_file, Language};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let parsed_file = parse_file("src/main.rs", Language::Rust).await?;
    
    println!("Found {} constructs", parsed_file.constructs.len());
    for construct in &parsed_file.constructs {
        if let Some(name) = &construct.name {
            println!("{}: {} (lines {}-{})", 
                construct.node_type, name, 
                construct.start_line, construct.end_line);
        }
    }
    
    Ok(())
}
```

### Parse an Entire Project

```rust
use tree_parser::{parse_directory, ParseOptions};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let options = ParseOptions::default();
    let project = parse_directory("./src", options).await?;
    
    println!("Processed {} files in {}ms", 
        project.total_files_processed, 
        project.processing_time_ms);
    
    // Print language distribution
    for (language, count) in &project.language_distribution {
        println!("{:?}: {} files", language, count);
    }
    
    Ok(())
}
```

### Search for Code Constructs

```rust
use tree_parser::{parse_file, search_by_node_type, Language};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let parsed_file = parse_file("example.py", Language::Python).await?;
    
    // Find all functions with names matching a pattern
    let functions = search_by_node_type(&parsed_file, "function_definition", Some(r"^test_.*"));
    
    for func in functions {
        println!("Test function: {}", func.name.unwrap_or_default());
        println!("Source: {}", func.source_code);
    }
    
    Ok(())
}
```

## Supported Languages

| Language   | Feature Flag    | File Extensions |
|------------|-----------------|----------------|
| Python     | `python`        | `.py`, `.pyw`, `.pyi` |
| Rust       | `rust_lang`     | `.rs` |
| JavaScript | `javascript`    | `.js`, `.mjs`, `.cjs` |
| TypeScript | `typescript`    | `.ts`, `.mts`, `.cts` |
| Java       | `java`          | `.java` |
| C          | `c`             | `.c`, `.h` |
| C++        | `cpp`           | `.cpp`, `.cc`, `.cxx`, `.hpp` |
| Go         | `go`            | `.go` |

## Advanced Usage

### Custom File Filtering

```rust
use tree_parser::{parse_directory_with_filter, FileFilter, ParseOptions, Language};
use std::path::Path;
use std::sync::Arc;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let filter = FileFilter {
        extensions: Some(vec!["rs".to_string(), "py".to_string()]),
        languages: Some(vec![Language::Rust, Language::Python]),
        min_size_bytes: Some(100),
        max_size_bytes: Some(1_000_000), // 1MB
        custom_predicate: Some(Arc::new(|path: &Path| {
            !path.to_string_lossy().contains("test")
        })),
    };
    
    let options = ParseOptions {
        max_concurrent_files: 8,
        include_hidden_files: false,
        max_file_size_mb: 5,
        ..Default::default()
    };
    
    let project = parse_directory_with_filter("./src", filter, options).await?;
    println!("Filtered parsing complete!");
    
    Ok(())
}
```

### Query-based Search

```rust
use tree_parser::{parse_file, search_by_query, Language};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let parsed_file = parse_file("example.py", Language::Python).await?;
    
    // Tree-sitter query to find all class definitions
    let query = r#"
        (class_definition
            name: (identifier) @class_name
            body: (block) @class_body)
    "#;
    
    let matches = search_by_query(&parsed_file, query, Language::Python)?;
    
    for m in matches {
        println!("Found class: {}", m.source_code);
    }
    
    Ok(())
}
```

## Configuration

### Parse Options

```rust
use tree_parser::{ParseOptions, LanguageDetection};

let options = ParseOptions {
    max_concurrent_files: 16,           // Concurrent file processing
    include_hidden_files: false,        // Skip hidden files
    max_file_size_mb: 10,              // Skip files larger than 10MB
    recursive: true,                    // Recursive directory traversal
    ignore_patterns: vec![              // Patterns to ignore
        "node_modules".to_string(),
        ".git".to_string(),
        "target".to_string(),
    ],
    language_detection: LanguageDetection::ByExtension,
    enable_caching: true,               // Enable internal caching
    thread_pool_size: Some(8),          // Custom thread pool size
};
```

## Error Handling

The library provides comprehensive error handling:

```rust
use tree_parser::{parse_file, Error, Language};

#[tokio::main]
async fn main() {
    match parse_file("nonexistent.py", Language::Python).await {
        Ok(parsed_file) => {
            println!("Successfully parsed file");
        }
        Err(Error::Io(msg)) => {
            eprintln!("IO error: {}", msg);
        }
        Err(Error::Parse(msg)) => {
            eprintln!("Parse error: {}", msg);
        }
        Err(Error::UnsupportedLanguage(lang)) => {
            eprintln!("Unsupported language: {}", lang);
        }
        Err(e) => {
            eprintln!("Other error: {}", e);
        }
    }
}
```

## Performance

The library is designed for high performance:

- **Concurrent Processing**: Uses tokio for async I/O and concurrent file processing
- **Memory Efficient**: Streaming processing for large codebases
- **Optimized Parsing**: Tree-sitter's incremental parsing capabilities
- **Configurable Limits**: Prevent resource exhaustion with configurable limits

## Contributing

Contributions are welcome! Please feel free to submit a Pull Request. For major changes, please open an issue first to discuss what you would like to change.

## License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.

## Acknowledgments

- [tree-sitter](https://tree-sitter.github.io/) for the excellent parsing framework
- The tree-sitter language grammar maintainers
- The Rust community for the amazing ecosystem