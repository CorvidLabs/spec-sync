use crate::exports::has_extension;
use crate::manifest;
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

/// Auto-detect source directories by first checking manifest files
/// (Cargo.toml, Package.swift, build.gradle.kts, package.json, etc.),
/// then falling back to scanning the project root for files with supported
/// language extensions. Returns directories relative to root.
pub fn detect_source_dirs(root: &Path) -> Vec<String> {
    // Try manifest-aware detection first
    let manifest_discovery = manifest::discover_from_manifests(root);
    if !manifest_discovery.source_dirs.is_empty() {
        let mut dirs = manifest_discovery.source_dirs;
        dirs.sort();
        dirs.dedup();
        return dirs;
    }

    // Fall back to directory scanning
    detect_source_dirs_by_scan(root)
}

/// Discover modules from manifest files (Package.swift, Cargo.toml, etc.).
/// Returns the manifest discovery result for use in module detection.
pub fn discover_manifest_modules(root: &Path) -> manifest::ManifestDiscovery {
    manifest::discover_from_manifests(root)
}

/// Scan-based source directory detection (fallback when no manifests found).
fn detect_source_dirs_by_scan(root: &Path) -> Vec<String> {
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

/// Known config keys in specsync.json (camelCase).
const KNOWN_JSON_KEYS: &[&str] = &[
    "specsDir",
    "sourceDirs",
    "schemaDir",
    "schemaPattern",
    "requiredSections",
    "excludeDirs",
    "excludePatterns",
    "sourceExtensions",
    "exportLevel",
    "modules",
    "aiProvider",
    "aiModel",
    "aiCommand",
    "aiApiKey",
    "aiBaseUrl",
    "aiTimeout",
    "rules",
    "taskArchiveDays",
    "github",
];

fn load_json_config(config_path: &Path, root: &Path) -> SpecSyncConfig {
    let content = match fs::read_to_string(config_path) {
        Ok(c) => c,
        Err(_) => return SpecSyncConfig::default(),
    };

    // Warn about unknown keys
    if let Ok(raw) = serde_json::from_str::<serde_json::Value>(&content)
        && let Some(obj) = raw.as_object()
    {
        for key in obj.keys() {
            if !KNOWN_JSON_KEYS.contains(&key.as_str()) {
                eprintln!("Warning: unknown key \"{key}\" in specsync.json (ignored)");
            }
        }
    }

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
    let mut current_section: Option<String> = None;

    for line in content.lines() {
        let line = line.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }

        // Track TOML section headers like [rules]
        if line.starts_with('[') && line.ends_with(']') {
            current_section = Some(line[1..line.len() - 1].trim().to_string());
            continue;
        }

        if let Some(eq_pos) = line.find('=') {
            let key = line[..eq_pos].trim();
            let value = line[eq_pos + 1..].trim();

            // Route to section-specific parsing
            if let Some(ref section) = current_section {
                match section.as_str() {
                    "rules" => {
                        parse_toml_rules_key(key, value, &mut config.rules);
                        continue;
                    }
                    "github" => {
                        parse_toml_github_key(key, value, &mut config);
                        continue;
                    }
                    _ => {
                        // Unknown section — skip silently
                        continue;
                    }
                }
            }

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
                "export_level" => {
                    let s = parse_toml_string(value);
                    match s.as_str() {
                        "type" => {
                            config.export_level = crate::types::ExportLevel::Type;
                        }
                        "member" => {
                            config.export_level = crate::types::ExportLevel::Member;
                        }
                        _ => eprintln!(
                            "Warning: unknown export_level \"{s}\" (expected \"type\" or \"member\")"
                        ),
                    }
                }
                "required_sections" => {
                    config.required_sections = parse_toml_string_array(value);
                }
                "task_archive_days" => {
                    if let Ok(n) = value.trim().parse::<u32>() {
                        config.task_archive_days = Some(n);
                    }
                }
                _ => {
                    eprintln!("Warning: unknown key \"{key}\" in .specsync.toml (ignored)");
                }
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

/// Parse a key=value pair inside a `[rules]` TOML section.
fn parse_toml_rules_key(key: &str, value: &str, rules: &mut crate::types::ValidationRules) {
    match key {
        "max_changelog_entries" => {
            if let Ok(n) = value.trim().parse::<usize>() {
                rules.max_changelog_entries = Some(n);
            }
        }
        "require_behavioral_examples" => {
            rules.require_behavioral_examples = Some(parse_toml_bool(value));
        }
        "min_invariants" => {
            if let Ok(n) = value.trim().parse::<usize>() {
                rules.min_invariants = Some(n);
            }
        }
        "max_spec_size_kb" => {
            if let Ok(n) = value.trim().parse::<usize>() {
                rules.max_spec_size_kb = Some(n);
            }
        }
        "require_depends_on" => {
            rules.require_depends_on = Some(parse_toml_bool(value));
        }
        _ => {
            eprintln!("Warning: unknown rule \"{key}\" in [rules] section (ignored)");
        }
    }
}

/// Parse a key=value pair inside a `[github]` TOML section.
fn parse_toml_github_key(key: &str, value: &str, config: &mut SpecSyncConfig) {
    let gh = config
        .github
        .get_or_insert_with(|| crate::types::GitHubConfig {
            repo: None,
            drift_labels: vec!["spec-drift".to_string()],
            verify_issues: true,
        });

    match key {
        "repo" => gh.repo = Some(parse_toml_string(value)),
        "drift_labels" => gh.drift_labels = parse_toml_string_array(value),
        "verify_issues" => gh.verify_issues = parse_toml_bool(value),
        _ => {
            eprintln!("Warning: unknown key \"{key}\" in [github] section (ignored)");
        }
    }
}

/// Parse a TOML boolean value.
fn parse_toml_bool(s: &str) -> bool {
    matches!(s.trim(), "true" | "yes" | "1")
}

/// Default schema pattern for SQL table extraction.
pub fn default_schema_pattern() -> &'static str {
    r"CREATE (?:VIRTUAL )?TABLE(?:\s+IF NOT EXISTS)?\s+(\w+)"
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    // --- TOML parsing helpers ---

    #[test]
    fn test_parse_toml_string_quoted() {
        assert_eq!(parse_toml_string("\"hello\""), "hello");
    }

    #[test]
    fn test_parse_toml_string_unquoted() {
        assert_eq!(parse_toml_string("bare_value"), "bare_value");
    }

    #[test]
    fn test_parse_toml_string_empty_quotes() {
        assert_eq!(parse_toml_string("\"\""), "");
    }

    #[test]
    fn test_parse_toml_string_with_whitespace() {
        assert_eq!(parse_toml_string("  \"trimmed\"  "), "trimmed");
    }

    #[test]
    fn test_parse_toml_string_array_basic() {
        let result = parse_toml_string_array("[\"a\", \"b\", \"c\"]");
        assert_eq!(result, vec!["a", "b", "c"]);
    }

    #[test]
    fn test_parse_toml_string_array_single() {
        let result = parse_toml_string_array("[\"only\"]");
        assert_eq!(result, vec!["only"]);
    }

    #[test]
    fn test_parse_toml_string_array_empty() {
        let result = parse_toml_string_array("[]");
        assert!(result.is_empty());
    }

    #[test]
    fn test_parse_toml_string_array_bare_value() {
        // When no brackets, treats as single-element array
        let result = parse_toml_string_array("\"single\"");
        assert_eq!(result, vec!["single"]);
    }

    #[test]
    fn test_parse_toml_bool_true_variants() {
        assert!(parse_toml_bool("true"));
        assert!(parse_toml_bool("yes"));
        assert!(parse_toml_bool("1"));
        assert!(parse_toml_bool("  true  "));
    }

    #[test]
    fn test_parse_toml_bool_false_variants() {
        assert!(!parse_toml_bool("false"));
        assert!(!parse_toml_bool("no"));
        assert!(!parse_toml_bool("0"));
        assert!(!parse_toml_bool("anything_else"));
    }

    // --- load_config ---

    #[test]
    fn test_load_config_no_config_file() {
        let tmp = TempDir::new().unwrap();
        let config = load_config(tmp.path());
        // Should return defaults with auto-detected source dirs
        assert_eq!(config.specs_dir, "specs");
        assert!(!config.source_dirs.is_empty());
    }

    #[test]
    fn test_load_config_json() {
        let tmp = TempDir::new().unwrap();
        fs::write(
            tmp.path().join("specsync.json"),
            r#"{"specsDir": "my-specs", "sourceDirs": ["lib", "app"]}"#,
        )
        .unwrap();

        let config = load_config(tmp.path());
        assert_eq!(config.specs_dir, "my-specs");
        assert_eq!(config.source_dirs, vec!["lib", "app"]);
    }

    #[test]
    fn test_load_config_toml() {
        let tmp = TempDir::new().unwrap();
        fs::write(
            tmp.path().join(".specsync.toml"),
            "specs_dir = \"custom-specs\"\nsource_dirs = [\"src\", \"lib\"]\n",
        )
        .unwrap();

        let config = load_config(tmp.path());
        assert_eq!(config.specs_dir, "custom-specs");
        assert_eq!(config.source_dirs, vec!["src", "lib"]);
    }

    #[test]
    fn test_load_config_json_takes_priority_over_toml() {
        let tmp = TempDir::new().unwrap();
        fs::write(
            tmp.path().join("specsync.json"),
            r#"{"specsDir": "from-json", "sourceDirs": ["src"]}"#,
        )
        .unwrap();
        fs::write(
            tmp.path().join(".specsync.toml"),
            "specs_dir = \"from-toml\"\nsource_dirs = [\"src\"]\n",
        )
        .unwrap();

        let config = load_config(tmp.path());
        assert_eq!(config.specs_dir, "from-json");
    }

    #[test]
    fn test_load_config_malformed_json_returns_defaults() {
        let tmp = TempDir::new().unwrap();
        fs::write(tmp.path().join("specsync.json"), "not valid json {{{").unwrap();

        let config = load_config(tmp.path());
        assert_eq!(config.specs_dir, "specs"); // default
    }

    #[test]
    fn test_load_config_json_without_source_dirs_auto_detects() {
        let tmp = TempDir::new().unwrap();
        fs::write(
            tmp.path().join("specsync.json"),
            r#"{"specsDir": "specs"}"#,
        )
        .unwrap();

        let config = load_config(tmp.path());
        // sourceDirs not in JSON, so it should auto-detect
        assert!(!config.source_dirs.is_empty());
    }

    // --- TOML config parsing ---

    #[test]
    fn test_toml_full_config() {
        let tmp = TempDir::new().unwrap();
        fs::write(
            tmp.path().join(".specsync.toml"),
            r#"
specs_dir = "specs"
source_dirs = ["src", "lib"]
schema_dir = "db/schema"
schema_pattern = "CREATE TABLE (\w+)"
exclude_dirs = ["__tests__"]
exclude_patterns = ["**/*.test.ts"]
source_extensions = [".ts", ".rs"]
export_level = "type"
ai_provider = "claude"
ai_model = "opus"
ai_timeout = 120
required_sections = ["Purpose", "Public API"]
task_archive_days = 30

[rules]
max_changelog_entries = 10
require_behavioral_examples = true
min_invariants = 2
max_spec_size_kb = 50
require_depends_on = true

[github]
repo = "CorvidLabs/spec-sync"
drift_labels = ["spec-drift", "needs-update"]
verify_issues = false
"#,
        )
        .unwrap();

        let config = load_config(tmp.path());
        assert_eq!(config.specs_dir, "specs");
        assert_eq!(config.source_dirs, vec!["src", "lib"]);
        assert_eq!(config.schema_dir.as_deref(), Some("db/schema"));
        assert_eq!(config.exclude_dirs, vec!["__tests__"]);
        assert_eq!(config.exclude_patterns, vec!["**/*.test.ts"]);
        assert_eq!(config.source_extensions, vec![".ts", ".rs"]);
        assert!(matches!(
            config.export_level,
            crate::types::ExportLevel::Type
        ));
        assert!(matches!(
            config.ai_provider,
            Some(crate::types::AiProvider::Claude)
        ));
        assert_eq!(config.ai_model.as_deref(), Some("opus"));
        assert_eq!(config.ai_timeout, Some(120));
        assert_eq!(
            config.required_sections,
            vec!["Purpose", "Public API"]
        );
        assert_eq!(config.task_archive_days, Some(30));

        // Rules
        assert_eq!(config.rules.max_changelog_entries, Some(10));
        assert_eq!(config.rules.require_behavioral_examples, Some(true));
        assert_eq!(config.rules.min_invariants, Some(2));
        assert_eq!(config.rules.max_spec_size_kb, Some(50));
        assert_eq!(config.rules.require_depends_on, Some(true));

        // GitHub
        let gh = config.github.unwrap();
        assert_eq!(gh.repo.as_deref(), Some("CorvidLabs/spec-sync"));
        assert_eq!(gh.drift_labels, vec!["spec-drift", "needs-update"]);
        assert!(!gh.verify_issues);
    }

    #[test]
    fn test_toml_comments_and_blank_lines() {
        let tmp = TempDir::new().unwrap();
        fs::write(
            tmp.path().join(".specsync.toml"),
            "# This is a comment\n\nspecs_dir = \"specs\"\n\n# Another comment\nsource_dirs = [\"src\"]\n",
        )
        .unwrap();

        let config = load_config(tmp.path());
        assert_eq!(config.specs_dir, "specs");
        assert_eq!(config.source_dirs, vec!["src"]);
    }

    #[test]
    fn test_toml_without_source_dirs_auto_detects() {
        let tmp = TempDir::new().unwrap();
        fs::write(
            tmp.path().join(".specsync.toml"),
            "specs_dir = \"specs\"\n",
        )
        .unwrap();

        let config = load_config(tmp.path());
        // source_dirs not specified, should auto-detect
        assert!(!config.source_dirs.is_empty());
    }

    // --- Source directory detection ---

    #[test]
    fn test_detect_source_dirs_empty_project() {
        let tmp = TempDir::new().unwrap();
        // Empty dir, no manifest, no source files -> fallback to "src"
        let dirs = detect_source_dirs(tmp.path());
        assert_eq!(dirs, vec!["src"]);
    }

    #[test]
    fn test_detect_source_dirs_with_src_dir() {
        let tmp = TempDir::new().unwrap();
        let src = tmp.path().join("src");
        fs::create_dir(&src).unwrap();
        fs::write(src.join("main.rs"), "fn main() {}").unwrap();

        let dirs = detect_source_dirs(tmp.path());
        assert!(dirs.contains(&"src".to_string()));
    }

    #[test]
    fn test_detect_source_dirs_ignores_node_modules() {
        let tmp = TempDir::new().unwrap();
        let nm = tmp.path().join("node_modules");
        fs::create_dir(&nm).unwrap();
        fs::write(nm.join("index.js"), "module.exports = {}").unwrap();

        let dirs = detect_source_dirs(tmp.path());
        assert!(!dirs.contains(&"node_modules".to_string()));
    }

    #[test]
    fn test_detect_source_dirs_root_source_files() {
        let tmp = TempDir::new().unwrap();
        fs::write(tmp.path().join("main.py"), "print('hello')").unwrap();

        let dirs = detect_source_dirs(tmp.path());
        assert_eq!(dirs, vec!["."]);
    }

    // --- default_schema_pattern ---

    #[test]
    fn test_default_schema_pattern_matches_create_table() {
        let pattern = regex::Regex::new(default_schema_pattern()).unwrap();
        assert!(pattern.is_match("CREATE TABLE users"));
        assert!(pattern.is_match("CREATE TABLE IF NOT EXISTS users"));
        assert!(pattern.is_match("CREATE VIRTUAL TABLE users_fts"));
    }

    #[test]
    fn test_default_schema_pattern_captures_table_name() {
        let pattern = regex::Regex::new(default_schema_pattern()).unwrap();
        let caps = pattern.captures("CREATE TABLE users (id INT)").unwrap();
        assert_eq!(&caps[1], "users");
    }
}
