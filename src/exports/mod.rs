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
        Language::TypeScript => {
            // Build a resolver that follows wildcard re-exports to sibling files
            let base_dir = file_path.parent().unwrap_or(Path::new(".")).to_path_buf();
            let resolver = move |import_path: &str| {
                resolve_ts_import(&base_dir, import_path)
            };
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
    };

    // Deduplicate preserving order
    let mut seen = std::collections::HashSet::new();
    symbols
        .into_iter()
        .filter(|s| seen.insert(s.clone()))
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
