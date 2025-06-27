//! Language detection and tree-sitter language loading

use crate::{Error, Language};
use std::path::Path;
use tree_sitter::Language as TSLanguage;

/// Get the tree-sitter language for a given Language enum
pub fn get_tree_sitter_language(language: &Language) -> Result<TSLanguage, Error> {
    match language {
        #[cfg(feature = "python")]
        Language::Python => Ok(tree_sitter_python::LANGUAGE.into()),
        #[cfg(feature = "rust_lang")]
        Language::Rust => Ok(tree_sitter_rust::LANGUAGE.into()),
        #[cfg(feature = "javascript")]
        Language::JavaScript => Ok(tree_sitter_javascript::LANGUAGE.into()),
        #[cfg(feature = "typescript")]
        Language::TypeScript => Ok(tree_sitter_typescript::LANGUAGE_TYPESCRIPT.into()),
        #[cfg(feature = "java")]
        Language::Java => Ok(tree_sitter_java::LANGUAGE.into()),
        #[cfg(feature = "c")]
        Language::C => Ok(tree_sitter_c::LANGUAGE.into()),
        #[cfg(feature = "cpp")]
        Language::Cpp => Ok(tree_sitter_cpp::LANGUAGE.into()),
        #[cfg(feature = "go")]
        Language::Go => Ok(tree_sitter_go::LANGUAGE.into()),
        _ => Err(Error::UnsupportedLanguage(format!("{:?}", language))),
    }
}

/// Detect language by file extension
pub fn detect_language_by_extension(file_path: &str) -> Option<Language> {
    let path = Path::new(file_path);
    let extension = path.extension()?.to_str()?.to_lowercase();
    
    match extension.as_str() {
        "py" | "pyw" | "pyi" => Some(Language::Python),
        "rs" => Some(Language::Rust),
        "js" | "mjs" | "cjs" => Some(Language::JavaScript),
        "ts" | "mts" | "cts" => Some(Language::TypeScript),
        "java" => Some(Language::Java),
        "c" | "h" => Some(Language::C),
        "cpp" | "cc" | "cxx" | "c++" | "hpp" | "hh" | "hxx" | "h++" => Some(Language::Cpp),
        "go" => Some(Language::Go),
        "cs" => Some(Language::CSharp),
        "php" | "phtml" | "php3" | "php4" | "php5" | "phps" => Some(Language::Php),
        "rb" | "rbw" => Some(Language::Ruby),
        "swift" => Some(Language::Swift),
        "kt" | "kts" => Some(Language::Kotlin),
        "scala" | "sc" => Some(Language::Scala),
        "hs" | "lhs" => Some(Language::Haskell),
        "lua" => Some(Language::Lua),
        "pl" | "pm" | "t" | "pod" => Some(Language::Perl),
        "r" | "R" => Some(Language::R),
        "sh" | "bash" | "zsh" | "fish" => Some(Language::Bash),
        "ps1" | "psm1" | "psd1" => Some(Language::PowerShell),
        "html" | "htm" | "xhtml" => Some(Language::Html),
        "css" => Some(Language::Css),
        "sql" => Some(Language::Sql),
        "json" => Some(Language::Json),
        "yaml" | "yml" => Some(Language::Yaml),
        "toml" => Some(Language::Toml),
        "xml" | "xsd" | "xsl" | "xslt" => Some(Language::Xml),
        _ => None,
    }
}

/// Detect language by shebang line
pub fn detect_language_by_shebang(content: &str) -> Option<Language> {
    let first_line = content.lines().next()?;
    if !first_line.starts_with("#!") {
        return None;
    }
    
    let shebang = first_line.to_lowercase();
    
    if shebang.contains("python") {
        Some(Language::Python)
    } else if shebang.contains("node") {
        Some(Language::JavaScript)
    } else if shebang.contains("bash") || shebang.contains("/bin/sh") {
        Some(Language::Bash)
    } else if shebang.contains("ruby") {
        Some(Language::Ruby)
    } else if shebang.contains("perl") {
        Some(Language::Perl)
    } else if shebang.contains("php") {
        Some(Language::Php)
    } else {
        None
    }
}

/// Detect language by file content patterns
pub fn detect_language_by_content(content: &str) -> Option<Language> {
    // Simple heuristics based on common patterns
    let content_lower = content.to_lowercase();
    
    // Check for specific language patterns
    if content_lower.contains("def ") && content_lower.contains("import ") {
        return Some(Language::Python);
    }
    
    if content_lower.contains("fn ") && content_lower.contains("use ") {
        return Some(Language::Rust);
    }
    
    if content_lower.contains("function ") && content_lower.contains("var ") {
        return Some(Language::JavaScript);
    }
    
    if content_lower.contains("public class ") && content_lower.contains("import ") {
        return Some(Language::Java);
    }
    
    if content_lower.contains("#include") && content_lower.contains("int main") {
        return Some(Language::C);
    }
    
    None
}

/// Combined language detection using multiple methods
pub fn detect_language(file_path: &str, content: Option<&str>) -> Option<Language> {
    // Try extension first
    if let Some(lang) = detect_language_by_extension(file_path) {
        return Some(lang);
    }
    
    // If content is provided, try shebang and content analysis
    if let Some(content) = content {
        if let Some(lang) = detect_language_by_shebang(content) {
            return Some(lang);
        }
        
        if let Some(lang) = detect_language_by_content(content) {
            return Some(lang);
        }
    }
    
    None
}

/// Get supported node types for a language
pub fn get_supported_node_types(language: &Language) -> Vec<String> {
    match language {
        Language::Python => vec![
            "function_definition".to_string(),
            "class_definition".to_string(),
            "import_statement".to_string(),
            "import_from_statement".to_string(),
            "assignment".to_string(),
            "decorated_definition".to_string(),
        ],
        Language::Rust => vec![
            "function_item".to_string(),
            "struct_item".to_string(),
            "enum_item".to_string(),
            "impl_item".to_string(),
            "trait_item".to_string(),
            "mod_item".to_string(),
            "use_declaration".to_string(),
            "const_item".to_string(),
            "static_item".to_string(),
        ],
        Language::JavaScript => vec![
            "function_declaration".to_string(),
            "function_expression".to_string(),
            "arrow_function".to_string(),
            "class_declaration".to_string(),
            "method_definition".to_string(),
            "variable_declaration".to_string(),
            "import_statement".to_string(),
            "export_statement".to_string(),
        ],
        Language::TypeScript => vec![
            "function_declaration".to_string(),
            "function_expression".to_string(),
            "arrow_function".to_string(),
            "class_declaration".to_string(),
            "interface_declaration".to_string(),
            "type_alias_declaration".to_string(),
            "method_definition".to_string(),
            "variable_declaration".to_string(),
            "import_statement".to_string(),
            "export_statement".to_string(),
        ],
        Language::Java => vec![
            "class_declaration".to_string(),
            "interface_declaration".to_string(),
            "method_declaration".to_string(),
            "constructor_declaration".to_string(),
            "field_declaration".to_string(),
            "import_declaration".to_string(),
            "package_declaration".to_string(),
        ],
        Language::C => vec![
            "function_definition".to_string(),
            "declaration".to_string(),
            "struct_specifier".to_string(),
            "union_specifier".to_string(),
            "enum_specifier".to_string(),
            "preproc_include".to_string(),
            "preproc_define".to_string(),
        ],
        Language::Cpp => vec![
            "function_definition".to_string(),
            "declaration".to_string(),
            "class_specifier".to_string(),
            "struct_specifier".to_string(),
            "union_specifier".to_string(),
            "enum_specifier".to_string(),
            "namespace_definition".to_string(),
            "preproc_include".to_string(),
            "preproc_define".to_string(),
        ],
        Language::Go => vec![
            "function_declaration".to_string(),
            "method_declaration".to_string(),
            "type_declaration".to_string(),
            "var_declaration".to_string(),
            "const_declaration".to_string(),
            "import_declaration".to_string(),
            "package_clause".to_string(),
        ],
        _ => vec![], // For unsupported languages
    }
}