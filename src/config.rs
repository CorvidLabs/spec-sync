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

/// Load config from specsync.json or .specsync.toml, falling back to defaults.
/// When no config file exists, auto-detects source directories.
///
/// Config file search order:
/// 1. `specsync.json` (JSON format)
/// 2. `.specsync.toml` (TOML format)
pub fn load_config(root: &Path) -> SpecSyncConfig {
    let json_path = root.join("specsync.json");
    let toml_path = root.join(".specsync.toml");

    if json_path.exists() {
        return load_json_config(&json_path, root);
    }

    if toml_path.exists() {
        return load_toml_config(&toml_path, root);
    }

    SpecSyncConfig {
        source_dirs: detect_source_dirs(root),
        ..Default::default()
    }
}

fn load_json_config(config_path: &Path, root: &Path) -> SpecSyncConfig {
    let content = match fs::read_to_string(config_path) {
        Ok(c) => c,
        Err(_) => return SpecSyncConfig::default(),
    };

    match serde_json::from_str::<SpecSyncConfig>(&content) {
        Ok(config) => {
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

/// Parse a TOML config file using zero-dependency parsing.
/// Supports the same fields as specsync.json but with TOML syntax:
///
/// ```toml
/// specs_dir = "specs"
/// source_dirs = ["src", "lib"]
/// schema_dir = "db/migrations"
/// exclude_dirs = ["__tests__"]
/// exclude_patterns = ["**/*.test.ts"]
/// ai_provider = "claude"
/// ai_model = "claude-sonnet-4-20250514"
/// ai_timeout = 120
/// required_sections = ["Purpose", "Public API"]
/// ```
fn load_toml_config(config_path: &Path, root: &Path) -> SpecSyncConfig {
    let content = match fs::read_to_string(config_path) {
        Ok(c) => c,
        Err(_) => return SpecSyncConfig::default(),
    };

    let mut config = SpecSyncConfig::default();
    let mut has_source_dirs = false;

    for line in content.lines() {
        let line = line.trim();
        if line.is_empty() || line.starts_with('#') || line.starts_with('[') {
            continue;
        }

        if let Some(eq_pos) = line.find('=') {
            let key = line[..eq_pos].trim();
            let value = line[eq_pos + 1..].trim();

            match key {
                "specs_dir" => config.specs_dir = parse_toml_string(value),
                "source_dirs" => {
                    config.source_dirs = parse_toml_string_array(value);
                    has_source_dirs = true;
                }
                "schema_dir" => config.schema_dir = Some(parse_toml_string(value)),
                "schema_pattern" => config.schema_pattern = Some(parse_toml_string(value)),
                "exclude_dirs" => config.exclude_dirs = parse_toml_string_array(value),
                "exclude_patterns" => config.exclude_patterns = parse_toml_string_array(value),
                "source_extensions" => config.source_extensions = parse_toml_string_array(value),
                "ai_provider" => {
                    let s = parse_toml_string(value);
                    config.ai_provider = crate::types::AiProvider::from_str_loose(&s);
                }
                "ai_model" => config.ai_model = Some(parse_toml_string(value)),
                "ai_command" => config.ai_command = Some(parse_toml_string(value)),
                "ai_api_key" => config.ai_api_key = Some(parse_toml_string(value)),
                "ai_base_url" => config.ai_base_url = Some(parse_toml_string(value)),
                "ai_timeout" => {
                    if let Ok(n) = value.trim().parse::<u64>() {
                        config.ai_timeout = Some(n);
                    }
                }
                "required_sections" => {
                    config.required_sections = parse_toml_string_array(value);
                }
                _ => {} // Ignore unknown keys
            }
        }
    }

    if !has_source_dirs {
        config.source_dirs = detect_source_dirs(root);
    }

    config
}

/// Parse a TOML string value: `"value"` -> `value`
fn parse_toml_string(s: &str) -> String {
    let s = s.trim();
    if s.starts_with('"') && s.ends_with('"') && s.len() >= 2 {
        s[1..s.len() - 1].to_string()
    } else {
        s.to_string()
    }
}

/// Parse a TOML array of strings: `["a", "b"]` -> vec!["a", "b"]
fn parse_toml_string_array(s: &str) -> Vec<String> {
    let s = s.trim();
    if !s.starts_with('[') || !s.ends_with(']') {
        return vec![parse_toml_string(s)];
    }
    let inner = &s[1..s.len() - 1];
    inner
        .split(',')
        .map(|item| parse_toml_string(item.trim()))
        .filter(|item| !item.is_empty())
        .collect()
}

/// Default schema pattern for SQL table extraction.
pub fn default_schema_pattern() -> &'static str {
    r"CREATE (?:VIRTUAL )?TABLE(?:\s+IF NOT EXISTS)?\s+(\w+)"
}
