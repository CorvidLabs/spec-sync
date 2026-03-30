mod csharp;
mod dart;
mod go;
mod java;
mod kotlin;
mod php;
mod python;
mod ruby;
mod rust_lang;
mod swift;
mod typescript;

use crate::types::{ExportLevel, Language};
use std::path::Path;

/// Extract exported symbol names from a source file, auto-detecting language.
/// Uses `ExportLevel::Member` (all symbols) for backwards compatibility.
pub fn get_exported_symbols(file_path: &Path) -> Vec<String> {
    get_exported_symbols_with_level(file_path, ExportLevel::Member)
}

/// Extract exported symbol names from a source file with configurable granularity.
/// When `level` is `Type`, only top-level type declarations are returned.
/// When `level` is `Member`, all public symbols are returned (default).
pub fn get_exported_symbols_with_level(file_path: &Path, level: ExportLevel) -> Vec<String> {
    let ext = file_path.extension().and_then(|e| e.to_str()).unwrap_or("");

    let lang = match Language::from_extension(ext) {
        Some(l) => l,
        None => return Vec::new(),
    };

    let content = match std::fs::read_to_string(file_path) {
        Ok(c) => c,
        Err(_) => return Vec::new(),
    };

    let symbols = match lang {
        Language::TypeScript => {
            // Build a resolver that follows wildcard re-exports to sibling files
            let base_dir = file_path.parent().unwrap_or(Path::new(".")).to_path_buf();
            let resolver = move |import_path: &str| resolve_ts_import(&base_dir, import_path);
            typescript::extract_exports_with_resolver(&content, Some(&resolver))
        }
        Language::Rust => rust_lang::extract_exports(&content),
        Language::Go => go::extract_exports(&content),
        Language::Python => python::extract_exports(&content),
        Language::Swift => swift::extract_exports(&content),
        Language::Kotlin => kotlin::extract_exports(&content),
        Language::Java => java::extract_exports(&content),
        Language::CSharp => csharp::extract_exports(&content),
        Language::Dart => dart::extract_exports(&content),
        Language::Php => php::extract_exports(&content),
        Language::Ruby => ruby::extract_exports(&content),
    };

    // If type-level granularity, filter to only type declarations
    let symbols = if level == ExportLevel::Type {
        filter_type_level_exports(&content, &symbols, lang)
    } else {
        symbols
    };

    // Deduplicate preserving order
    let mut seen = std::collections::HashSet::new();
    symbols
        .into_iter()
        .filter(|s| seen.insert(s.clone()))
        .collect()
}

/// Filter symbols to only include type-level declarations (class, struct, enum, etc.).
/// This removes individual functions, variables, constants, and properties.
fn filter_type_level_exports(content: &str, symbols: &[String], lang: Language) -> Vec<String> {
    use regex::Regex;

    let type_pattern = match lang {
        Language::TypeScript => {
            // class, interface, type, enum — but not function, const, var, let
            Regex::new(
                r"(?m)export\s+(?:default\s+)?(?:abstract\s+)?(?:class|interface|type|enum)\s+(\w+)",
            )
            .ok()
        }
        Language::Rust => {
            Regex::new(r"(?m)pub(?:\(crate\))?\s+(?:struct|enum|trait|type|mod)\s+(\w+)").ok()
        }
        Language::Go => {
            // Go: type X struct/interface
            Regex::new(r"(?m)^type\s+([A-Z]\w*)\s+(?:struct|interface)").ok()
        }
        Language::Python => {
            Regex::new(r"(?m)^class\s+(\w+)").ok()
        }
        Language::Swift => {
            Regex::new(
                r"(?m)(?:public|open)\s+(?:final\s+)?(?:class|struct|enum|protocol|actor)\s+(\w+)",
            )
            .ok()
        }
        Language::Kotlin => {
            Regex::new(
                r"(?m)(?:public\s+|open\s+|abstract\s+|sealed\s+)*(?:class|interface|enum\s+class|object|data\s+class)\s+(\w+)",
            )
            .ok()
        }
        Language::Java => {
            Regex::new(
                r"(?m)(?:public\s+)?(?:abstract\s+|final\s+)?(?:class|interface|enum|record)\s+(\w+)",
            )
            .ok()
        }
        Language::CSharp => {
            Regex::new(
                r"(?m)(?:public\s+)?(?:abstract\s+|sealed\s+|static\s+)?(?:class|interface|enum|struct|record)\s+(\w+)",
            )
            .ok()
        }
        Language::Dart => {
            Regex::new(r"(?m)(?:abstract\s+)?class\s+(\w+)|(?m)enum\s+(\w+)").ok()
        }
        Language::Php => {
            Regex::new(
                r"(?m)(?:abstract\s+|final\s+)?(?:readonly\s+)?(?:class|interface|trait|enum)\s+(\w+)",
            )
            .ok()
        }
        Language::Ruby => {
            Regex::new(r"(?m)(?:class|module)\s+([A-Z]\w*)").ok()
        }
    };

    let type_names: std::collections::HashSet<String> = match type_pattern {
        Some(re) => re
            .captures_iter(content)
            .filter_map(|caps| {
                caps.get(1)
                    .or_else(|| caps.get(2))
                    .map(|m| m.as_str().to_string())
            })
            .collect(),
        None => return symbols.to_vec(),
    };

    symbols
        .iter()
        .filter(|s| type_names.contains(s.as_str()))
        .cloned()
        .collect()
}

/// Resolve a TypeScript/JavaScript relative import to file content.
/// Tries common extensions: .ts, .tsx, .js, .jsx, /index.ts, /index.js
fn resolve_ts_import(base_dir: &Path, import_path: &str) -> Option<String> {
    // Only resolve relative imports
    if !import_path.starts_with('.') {
        return None;
    }

    let target = base_dir.join(import_path);

    // Try exact path first (might already have extension)
    if target.is_file() {
        return std::fs::read_to_string(&target).ok();
    }

    // Try common extensions
    for ext in &[".ts", ".tsx", ".js", ".jsx", ".mts", ".cts"] {
        let with_ext = target.with_extension(ext.trim_start_matches('.'));
        if with_ext.is_file() {
            return std::fs::read_to_string(&with_ext).ok();
        }
    }

    // Try as directory with index file
    for index in &["index.ts", "index.tsx", "index.js", "index.jsx"] {
        let index_path = target.join(index);
        if index_path.is_file() {
            return std::fs::read_to_string(&index_path).ok();
        }
    }

    None
}

/// Well-known test directory names (case-insensitive check).
const TEST_DIR_NAMES: &[&str] = &[
    "tests",
    "test",
    "__tests__",
    "spec",
    "specs",
    "testing",
    "uitests",
    "unittests",
    "integrationtests",
    "testcases",
    "fixtures",
    "mocks",
    "stubs",
    "fakes",
];

/// Check if a file is a test file based on language conventions and path.
pub fn is_test_file(file_path: &Path) -> bool {
    let ext = file_path.extension().and_then(|e| e.to_str()).unwrap_or("");

    let lang = match Language::from_extension(ext) {
        Some(l) => l,
        None => return false,
    };

    let name = file_path.file_name().and_then(|n| n.to_str()).unwrap_or("");

    // Check filename patterns
    for pattern in lang.test_patterns() {
        if name.ends_with(pattern) || name.starts_with(pattern) {
            return true;
        }
    }

    // Check if any ancestor directory is a test directory
    for component in file_path.components() {
        if let std::path::Component::Normal(dir) = component {
            let dir_lower = dir.to_string_lossy().to_lowercase();
            if TEST_DIR_NAMES.contains(&dir_lower.as_str()) {
                return true;
            }
        }
    }

    false
}

/// Check if a file extension is a supported source file.
pub fn is_source_file(file_path: &Path) -> bool {
    let ext = file_path.extension().and_then(|e| e.to_str()).unwrap_or("");

    Language::from_extension(ext).is_some()
}

/// Check if a file extension matches a specific set of allowed extensions.
pub fn has_extension(file_path: &Path, extensions: &[String]) -> bool {
    if extensions.is_empty() {
        return is_source_file(file_path);
    }
    let ext = file_path.extension().and_then(|e| e.to_str()).unwrap_or("");
    extensions.iter().any(|e| e == ext)
}
