mod csharp;
mod dart;
mod go;
mod java;
mod kotlin;
mod python;
mod rust_lang;
mod swift;
mod typescript;

use crate::types::Language;
use std::path::Path;

/// Extract exported symbol names from a source file, auto-detecting language.
pub fn get_exported_symbols(file_path: &Path) -> Vec<String> {
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
        Language::TypeScript => typescript::extract_exports(&content),
        Language::Rust => rust_lang::extract_exports(&content),
        Language::Go => go::extract_exports(&content),
        Language::Python => python::extract_exports(&content),
        Language::Swift => swift::extract_exports(&content),
        Language::Kotlin => kotlin::extract_exports(&content),
        Language::Java => java::extract_exports(&content),
        Language::CSharp => csharp::extract_exports(&content),
        Language::Dart => dart::extract_exports(&content),
    };

    // Deduplicate preserving order
    let mut seen = std::collections::HashSet::new();
    symbols
        .into_iter()
        .filter(|s| seen.insert(s.clone()))
        .collect()
}

/// Check if a file is a test file based on language conventions.
pub fn is_test_file(file_path: &Path) -> bool {
    let ext = file_path.extension().and_then(|e| e.to_str()).unwrap_or("");

    let lang = match Language::from_extension(ext) {
        Some(l) => l,
        None => return false,
    };

    let name = file_path.file_name().and_then(|n| n.to_str()).unwrap_or("");

    for pattern in lang.test_patterns() {
        if name.ends_with(pattern) || name.starts_with(pattern) {
            return true;
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
