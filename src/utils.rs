//! Utility functions for the tree parser library

use crate::Language;

/// Check if a file extension is supported
pub fn is_supported_extension(extension: &str) -> bool {
    crate::languages::detect_language_by_extension(&format!("file.{}", extension)).is_some()
}

/// Get all supported file extensions
pub fn get_supported_extensions() -> Vec<String> {
    vec![
        "py".to_string(), "pyw".to_string(), "pyi".to_string(),
        "rs".to_string(),
        "js".to_string(), "mjs".to_string(), "cjs".to_string(),
        "ts".to_string(), "mts".to_string(), "cts".to_string(),
        "java".to_string(),
        "c".to_string(), "h".to_string(),
        "cpp".to_string(), "cc".to_string(), "cxx".to_string(), "c++".to_string(),
        "hpp".to_string(), "hh".to_string(), "hxx".to_string(), "h++".to_string(),
        "go".to_string(),
        "cs".to_string(),
        "php".to_string(), "phtml".to_string(), "php3".to_string(), "php4".to_string(), "php5".to_string(), "phps".to_string(),
        "rb".to_string(), "rbw".to_string(),
        "swift".to_string(),
        "kt".to_string(), "kts".to_string(),
        "scala".to_string(), "sc".to_string(),
        "hs".to_string(), "lhs".to_string(),
        "lua".to_string(),
        "pl".to_string(), "pm".to_string(), "t".to_string(), "pod".to_string(),
        "r".to_string(), "R".to_string(),
        "sh".to_string(), "bash".to_string(), "zsh".to_string(), "fish".to_string(),
        "ps1".to_string(), "psm1".to_string(), "psd1".to_string(),
        "html".to_string(), "htm".to_string(), "xhtml".to_string(),
        "css".to_string(),
        "sql".to_string(),
        "json".to_string(),
        "yaml".to_string(), "yml".to_string(),
        "toml".to_string(),
        "xml".to_string(), "xsd".to_string(), "xsl".to_string(), "xslt".to_string(),
    ]
}

/// Get language from string
pub fn language_from_string(lang_str: &str) -> Option<Language> {
    match lang_str.to_lowercase().as_str() {
        "python" | "py" => Some(Language::Python),
        "rust" | "rs" => Some(Language::Rust),
        "javascript" | "js" => Some(Language::JavaScript),
        "typescript" | "ts" => Some(Language::TypeScript),
        "java" => Some(Language::Java),
        "c" => Some(Language::C),
        "cpp" | "c++" | "cxx" => Some(Language::Cpp),
        "go" | "golang" => Some(Language::Go),
        "csharp" | "c#" | "cs" => Some(Language::CSharp),
        "php" => Some(Language::Php),
        "ruby" | "rb" => Some(Language::Ruby),
        "swift" => Some(Language::Swift),
        "kotlin" | "kt" => Some(Language::Kotlin),
        "scala" => Some(Language::Scala),
        "haskell" | "hs" => Some(Language::Haskell),
        "lua" => Some(Language::Lua),
        "perl" | "pl" => Some(Language::Perl),
        "r" => Some(Language::R),
        "bash" | "sh" => Some(Language::Bash),
        "powershell" | "ps1" => Some(Language::PowerShell),
        "html" => Some(Language::Html),
        "css" => Some(Language::Css),
        "sql" => Some(Language::Sql),
        "json" => Some(Language::Json),
        "yaml" | "yml" => Some(Language::Yaml),
        "toml" => Some(Language::Toml),
        "xml" => Some(Language::Xml),
        _ => None,
    }
}

/// Convert language to string
pub fn language_to_string(language: &Language) -> String {
    match language {
        Language::Python => "Python".to_string(),
        Language::Rust => "Rust".to_string(),
        Language::JavaScript => "JavaScript".to_string(),
        Language::TypeScript => "TypeScript".to_string(),
        Language::Java => "Java".to_string(),
        Language::C => "C".to_string(),
        Language::Cpp => "C++".to_string(),
        Language::Go => "Go".to_string(),
        Language::CSharp => "C#".to_string(),
        Language::Php => "PHP".to_string(),
        Language::Ruby => "Ruby".to_string(),
        Language::Swift => "Swift".to_string(),
        Language::Kotlin => "Kotlin".to_string(),
        Language::Scala => "Scala".to_string(),
        Language::Haskell => "Haskell".to_string(),
        Language::Lua => "Lua".to_string(),
        Language::Perl => "Perl".to_string(),
        Language::R => "R".to_string(),
        Language::Bash => "Bash".to_string(),
        Language::PowerShell => "PowerShell".to_string(),
        Language::Html => "HTML".to_string(),
        Language::Css => "CSS".to_string(),
        Language::Sql => "SQL".to_string(),
        Language::Json => "JSON".to_string(),
        Language::Yaml => "YAML".to_string(),
        Language::Toml => "TOML".to_string(),
        Language::Xml => "XML".to_string(),
    }
}

/// Format file size in human readable format
pub fn format_file_size(bytes: usize) -> String {
    const UNITS: &[&str] = &["B", "KB", "MB", "GB", "TB"];
    let mut size = bytes as f64;
    let mut unit_index = 0;
    
    while size >= 1024.0 && unit_index < UNITS.len() - 1 {
        size /= 1024.0;
        unit_index += 1;
    }
    
    if unit_index == 0 {
        format!("{} {}", bytes, UNITS[unit_index])
    } else {
        format!("{:.2} {}", size, UNITS[unit_index])
    }
}

/// Format duration in human readable format
pub fn format_duration(ms: u64) -> String {
    if ms < 1000 {
        format!("{}ms", ms)
    } else if ms < 60_000 {
        format!("{:.2}s", ms as f64 / 1000.0)
    } else {
        let minutes = ms / 60_000;
        let seconds = (ms % 60_000) as f64 / 1000.0;
        format!("{}m {:.2}s", minutes, seconds)
    }
}

/// Validate file path
pub fn is_valid_file_path(path: &str) -> bool {
    std::path::Path::new(path).exists()
}

/// Validate directory path
pub fn is_valid_directory_path(path: &str) -> bool {
    let path = std::path::Path::new(path);
    path.exists() && path.is_dir()
}

/// Get file extension from path
pub fn get_file_extension(path: &str) -> Option<String> {
    std::path::Path::new(path)
        .extension()
        .and_then(|ext| ext.to_str())
        .map(|ext| ext.to_lowercase())
}

/// Get file name without extension
pub fn get_file_name_without_extension(path: &str) -> Option<String> {
    std::path::Path::new(path)
        .file_stem()
        .and_then(|name| name.to_str())
        .map(|name| name.to_string())
}

/// Check if path matches any of the ignore patterns
pub fn matches_ignore_patterns(path: &str, patterns: &[String]) -> bool {
    for pattern in patterns {
        if path.contains(pattern) {
            return true;
        }
    }
    false
}

/// Sanitize file path for safe usage
pub fn sanitize_path(path: &str) -> String {
    path.replace("..", "")
        .replace("//", "/")
        .trim_start_matches('/')
        .to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_format_file_size() {
        assert_eq!(format_file_size(512), "512 B");
        assert_eq!(format_file_size(1024), "1.00 KB");
        assert_eq!(format_file_size(1536), "1.50 KB");
        assert_eq!(format_file_size(1048576), "1.00 MB");
    }

    #[test]
    fn test_format_duration() {
        assert_eq!(format_duration(500), "500ms");
        assert_eq!(format_duration(1500), "1.50s");
        assert_eq!(format_duration(65000), "1m 5.00s");
    }

    #[test]
    fn test_language_conversion() {
        assert_eq!(language_from_string("python"), Some(Language::Python));
        assert_eq!(language_from_string("rust"), Some(Language::Rust));
        assert_eq!(language_from_string("invalid"), None);
        
        assert_eq!(language_to_string(&Language::Python), "Python");
        assert_eq!(language_to_string(&Language::Rust), "Rust");
    }

    #[test]
    fn test_file_extension() {
        assert_eq!(get_file_extension("test.py"), Some("py".to_string()));
        assert_eq!(get_file_extension("test.RS"), Some("rs".to_string()));
        assert_eq!(get_file_extension("test"), None);
    }

    #[test]
    fn test_supported_extensions() {
        assert!(is_supported_extension("py"));
        assert!(is_supported_extension("rs"));
        assert!(is_supported_extension("js"));
        assert!(!is_supported_extension("xyz"));
    }
}