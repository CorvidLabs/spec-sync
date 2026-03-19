use crate::exports::has_extension;
use crate::types::SpecSyncConfig;
use std::collections::HashSet;
use std::fs;
use std::path::Path;
use walkdir::WalkDir;

/// Directories that should never be treated as source directories.
const IGNORED_DIRS: &[&str] = &[
    "node_modules",
    ".git",
    ".hg",
    ".svn",
    "dist",
    "build",
    "out",
    "target",
    "vendor",
    ".next",
    ".nuxt",
    ".output",
    ".cache",
    ".turbo",
    "coverage",
    "__pycache__",
    ".mypy_cache",
    ".pytest_cache",
    ".tox",
    ".venv",
    "venv",
    "env",
    ".env",
    ".idea",
    ".vscode",
    ".DS_Store",
    "specs",
    "docs",
    "doc",
    ".github",
    ".gitlab",
    "migrations",
    "Pods",
    ".dart_tool",
    ".gradle",
    "bin",
    "obj",
];

/// Auto-detect source directories by scanning the project root for files
/// with supported language extensions. Returns directories relative to root.
pub fn detect_source_dirs(root: &Path) -> Vec<String> {
    let ignored: HashSet<&str> = IGNORED_DIRS.iter().copied().collect();
    let mut source_dirs: Vec<String> = Vec::new();
    let mut has_root_source_files = false;

    // Check immediate children of root
    let entries = match fs::read_dir(root) {
        Ok(e) => e,
        Err(_) => return vec!["src".to_string()],
    };

    for entry in entries.flatten() {
        let name = entry.file_name().to_string_lossy().to_string();

        // Skip hidden dirs and ignored dirs
        if name.starts_with('.') || ignored.contains(name.as_str()) {
            continue;
        }

        let path = entry.path();

        if path.is_dir() {
            // Check if this directory contains any source files (scan up to 3 levels deep)
            if dir_contains_source_files(&path, &ignored, 3) {
                source_dirs.push(name);
            }
        } else if path.is_file() && has_extension(&path, &[]) {
            // Source file directly in root
            has_root_source_files = true;
        }
    }

    // If source files exist directly in root, add "." as a source dir
    if has_root_source_files && source_dirs.is_empty() {
        return vec![".".to_string()];
    }

    if source_dirs.is_empty() {
        // Fallback to "src" if nothing detected
        return vec!["src".to_string()];
    }

    source_dirs.sort();
    source_dirs
}

/// Check if a directory contains source files, scanning up to `max_depth` levels.
fn dir_contains_source_files(dir: &Path, ignored: &HashSet<&str>, max_depth: usize) -> bool {
    for entry in WalkDir::new(dir)
        .max_depth(max_depth)
        .into_iter()
        .filter_entry(|e| {
            if e.file_type().is_dir() {
                let name = e.file_name().to_str().unwrap_or("");
                !name.starts_with('.') && !ignored.contains(name)
            } else {
                true
            }
        })
        .filter_map(|e| e.ok())
    {
        let path = entry.path();
        if path.is_file() && has_extension(path, &[]) {
            return true;
        }
    }
    false
}

/// Load specsync.json from the project root, falling back to defaults.
/// When no config file exists, auto-detects source directories.
pub fn load_config(root: &Path) -> SpecSyncConfig {
    let config_path = root.join("specsync.json");

    if !config_path.exists() {
        return SpecSyncConfig {
            source_dirs: detect_source_dirs(root),
            ..Default::default()
        };
    }

    let content = match fs::read_to_string(&config_path) {
        Ok(c) => c,
        Err(_) => return SpecSyncConfig::default(),
    };

    match serde_json::from_str::<SpecSyncConfig>(&content) {
        Ok(config) => {
            // If sourceDirs was not explicitly set in the config file (still default ["src"]),
            // check if we should auto-detect. We do this by checking if the raw JSON
            // contains a "sourceDirs" key.
            if !content.contains("\"sourceDirs\"") {
                let mut config = config;
                config.source_dirs = detect_source_dirs(root);
                return config;
            }
            config
        }
        Err(e) => {
            eprintln!("Warning: failed to parse specsync.json: {e}");
            SpecSyncConfig::default()
        }
    }
}

/// Default schema pattern for SQL table extraction.
pub fn default_schema_pattern() -> &'static str {
    r"CREATE (?:VIRTUAL )?TABLE(?:\s+IF NOT EXISTS)?\s+(\w+)"
}
