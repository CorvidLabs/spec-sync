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

/// Load config from TOML or JSON, falling back to defaults.
/// When no config file exists, auto-detects source directories.
///
/// Config file search order (v4 first, then legacy):
/// Load config from a specific file path (JSON or TOML based on extension).
/// Used by migration to convert a known source file rather than relying on precedence.
pub fn load_config_from_path(config_path: &Path, root: &Path) -> SpecSyncConfig {
    let ext = config_path
        .extension()
        .and_then(|e| e.to_str())
        .unwrap_or("");
    match ext {
        "toml" => load_toml_config(config_path, root),
        _ => load_json_config(config_path, root),
    }
}

/// 1. `.specsync/config.toml` (v4 TOML — canonical)
/// 2. `.specsync/config.json` (v4 JSON — pre-TOML migration)
/// 3. `.specsync.toml` (legacy root TOML)
/// 4. `specsync.json` (legacy root JSON)
pub fn load_config(root: &Path) -> SpecSyncConfig {
    let v4_toml = root.join(".specsync/config.toml");
    let v4_json = root.join(".specsync/config.json");
    let legacy_toml = root.join(".specsync.toml");
    let legacy_json = root.join("specsync.json");

    if v4_toml.exists() {
        return load_toml_config(&v4_toml, root);
    }

    if v4_json.exists() {
        return load_json_config(&v4_json, root);
    }

    if legacy_toml.exists() {
        return load_toml_config(&legacy_toml, root);
    }

    if legacy_json.exists() {
        return load_json_config(&legacy_json, root);
    }

    SpecSyncConfig {
        source_dirs: detect_source_dirs(root),
        ..Default::default()
    }
}

/// Detect whether this project is using a legacy 3.x layout.
/// Returns true if root-level config files exist without a .specsync/version stamp.
pub fn is_legacy_layout(root: &Path) -> bool {
    let has_root_json = root.join("specsync.json").exists();
    let has_root_toml = root.join(".specsync.toml").exists();
    let has_root_registry = root.join("specsync-registry.toml").exists();
    let has_v4_version = root.join(".specsync/version").exists();

    (has_root_json || has_root_toml || has_root_registry) && !has_v4_version
}

/// Escape a string value for safe embedding in a TOML quoted string.
fn toml_escape(s: &str) -> String {
    s.replace('\\', "\\\\")
        .replace('"', "\\\"")
        .replace('\n', "\\n")
        .replace('\r', "\\r")
        .replace('\t', "\\t")
}

/// Serialize a SpecSyncConfig to TOML format string.
pub fn config_to_toml(config: &SpecSyncConfig) -> String {
    let mut lines: Vec<String> = Vec::new();
    lines.push("# spec-sync v4 configuration".to_string());
    lines.push("# Docs: https://github.com/CorvidLabs/spec-sync".to_string());
    lines.push(String::new());

    // Core settings
    lines.push(format!(
        "specs_dir = \"{}\"",
        toml_escape(&config.specs_dir)
    ));

    if !config.source_dirs.is_empty() {
        lines.push(format!(
            "source_dirs = [{}]",
            config
                .source_dirs
                .iter()
                .map(|s| format!("\"{}\"", toml_escape(s)))
                .collect::<Vec<_>>()
                .join(", ")
        ));
    }

    if let Some(ref schema_dir) = config.schema_dir {
        lines.push(format!("schema_dir = \"{}\"", toml_escape(schema_dir)));
    }
    if let Some(ref schema_pattern) = config.schema_pattern {
        lines.push(format!(
            "schema_pattern = \"{}\"",
            toml_escape(schema_pattern)
        ));
    }

    if !config.exclude_dirs.is_empty() {
        lines.push(format!(
            "exclude_dirs = [{}]",
            config
                .exclude_dirs
                .iter()
                .map(|s| format!("\"{}\"", toml_escape(s)))
                .collect::<Vec<_>>()
                .join(", ")
        ));
    }
    if !config.exclude_patterns.is_empty() {
        lines.push(format!(
            "exclude_patterns = [{}]",
            config
                .exclude_patterns
                .iter()
                .map(|s| format!("\"{}\"", toml_escape(s)))
                .collect::<Vec<_>>()
                .join(", ")
        ));
    }
    if !config.source_extensions.is_empty() {
        lines.push(format!(
            "source_extensions = [{}]",
            config
                .source_extensions
                .iter()
                .map(|s| format!("\"{}\"", toml_escape(s)))
                .collect::<Vec<_>>()
                .join(", ")
        ));
    }

    if !config.required_sections.is_empty() {
        lines.push(format!(
            "required_sections = [{}]",
            config
                .required_sections
                .iter()
                .map(|s| format!("\"{}\"", toml_escape(s)))
                .collect::<Vec<_>>()
                .join(", ")
        ));
    }

    // Export level
    match config.export_level {
        crate::types::ExportLevel::Type => lines.push("export_level = \"type\"".to_string()),
        crate::types::ExportLevel::Member => {} // default, omit
    }

    // Enforcement
    match config.enforcement {
        crate::types::EnforcementMode::Warn => {} // default, omit
        crate::types::EnforcementMode::EnforceNew => {
            lines.push("enforcement = \"enforce-new\"".to_string());
        }
        crate::types::EnforcementMode::Strict => {
            lines.push("enforcement = \"strict\"".to_string());
        }
    }

    // AI settings
    if let Some(ref provider) = config.ai_provider {
        let name = match provider {
            crate::types::AiProvider::Claude => "claude",
            crate::types::AiProvider::Cursor => "cursor",
            crate::types::AiProvider::Copilot => "copilot",
            crate::types::AiProvider::Ollama => "ollama",
            crate::types::AiProvider::Anthropic => "anthropic",
            crate::types::AiProvider::OpenAi => "openai",
            crate::types::AiProvider::Gemini => "gemini",
            crate::types::AiProvider::DeepSeek => "deepseek",
            crate::types::AiProvider::Groq => "groq",
            crate::types::AiProvider::Mistral => "mistral",
            crate::types::AiProvider::XAi => "xai",
            crate::types::AiProvider::Together => "together",
            crate::types::AiProvider::Custom => "custom",
        };
        lines.push(format!("ai_provider = \"{name}\""));
    }
    if let Some(ref model) = config.ai_model {
        lines.push(format!("ai_model = \"{}\"", toml_escape(model)));
    }
    if let Some(ref cmd) = config.ai_command {
        lines.push(format!("ai_command = \"{}\"", toml_escape(cmd)));
    }
    if config.ai_api_key.is_some() {
        eprintln!("[warn] ai_api_key found in config but NOT written to config.toml.");
        eprintln!("       Set the AI_API_KEY environment variable instead.");
    }
    if let Some(ref url) = config.ai_base_url {
        lines.push(format!("ai_base_url = \"{}\"", toml_escape(url)));
    }
    if let Some(timeout) = config.ai_timeout {
        lines.push(format!("ai_timeout = {timeout}"));
    }

    if let Some(days) = config.task_archive_days {
        lines.push(format!("task_archive_days = {days}"));
    }

    // Rules section
    let rules = &config.rules;
    let has_rules = rules.max_changelog_entries.is_some()
        || rules.require_behavioral_examples.is_some()
        || rules.min_invariants.is_some()
        || rules.max_spec_size_kb.is_some()
        || rules.require_depends_on.is_some();

    if has_rules {
        lines.push(String::new());
        lines.push("[rules]".to_string());
        if let Some(n) = rules.max_changelog_entries {
            lines.push(format!("max_changelog_entries = {n}"));
        }
        if let Some(b) = rules.require_behavioral_examples {
            lines.push(format!("require_behavioral_examples = {b}"));
        }
        if let Some(n) = rules.min_invariants {
            lines.push(format!("min_invariants = {n}"));
        }
        if let Some(n) = rules.max_spec_size_kb {
            lines.push(format!("max_spec_size_kb = {n}"));
        }
        if let Some(b) = rules.require_depends_on {
            lines.push(format!("require_depends_on = {b}"));
        }
    }

    // GitHub section
    if let Some(ref gh) = config.github {
        lines.push(String::new());
        lines.push("[github]".to_string());
        if let Some(ref repo) = gh.repo {
            lines.push(format!("repo = \"{}\"", toml_escape(repo)));
        }
        if !gh.drift_labels.is_empty() {
            lines.push(format!(
                "drift_labels = [{}]",
                gh.drift_labels
                    .iter()
                    .map(|s| format!("\"{}\"", toml_escape(s)))
                    .collect::<Vec<_>>()
                    .join(", ")
            ));
        }
        if !gh.verify_issues {
            lines.push(format!("verify_issues = {}", gh.verify_issues));
        }
    }

    // Lifecycle section
    let lc = &config.lifecycle;
    let has_lifecycle = !lc.guards.is_empty()
        || !lc.track_history
        || !lc.max_age.is_empty()
        || !lc.allowed_statuses.is_empty();

    if has_lifecycle {
        lines.push(String::new());
        lines.push("[lifecycle]".to_string());
        if !lc.track_history {
            lines.push(format!("track_history = {}", lc.track_history));
        }
        if !lc.allowed_statuses.is_empty() {
            lines.push(format!(
                "allowed_statuses = [{}]",
                lc.allowed_statuses
                    .iter()
                    .map(|s| format!("\"{}\"", toml_escape(s)))
                    .collect::<Vec<_>>()
                    .join(", ")
            ));
        }
        if !lc.max_age.is_empty() {
            lines.push(String::new());
            lines.push("[lifecycle.max_age]".to_string());
            for (status, days) in &lc.max_age {
                lines.push(format!("{status} = {days}"));
            }
        }
        if !lc.guards.is_empty() {
            for (transition, guard) in &lc.guards {
                lines.push(String::new());
                lines.push(format!("[lifecycle.guards.\"{transition}\"]"));
                if let Some(score) = guard.min_score {
                    lines.push(format!("min_score = {score}"));
                }
                if !guard.require_sections.is_empty() {
                    lines.push(format!(
                        "require_sections = [{}]",
                        guard
                            .require_sections
                            .iter()
                            .map(|s| format!("\"{}\"", toml_escape(s)))
                            .collect::<Vec<_>>()
                            .join(", ")
                    ));
                }
                if let Some(no_stale) = guard.no_stale {
                    lines.push(format!("no_stale = {no_stale}"));
                }
                if let Some(threshold) = guard.stale_threshold {
                    lines.push(format!("stale_threshold = {threshold}"));
                }
                if let Some(ref msg) = guard.message {
                    lines.push(format!("message = \"{}\"", toml_escape(msg)));
                }
            }
        }
    }

    lines.push(String::new()); // trailing newline
    lines.join("\n")
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
    "customRules",
    "taskArchiveDays",
    "github",
    "enforcement",
    "lifecycle",
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
                    "lifecycle" => {
                        parse_toml_lifecycle_key(key, value, &mut config.lifecycle);
                        continue;
                    }
                    s if s.starts_with("lifecycle.") => {
                        parse_toml_lifecycle_nested(s, key, value, &mut config.lifecycle);
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
                "enforcement" => {
                    let s = parse_toml_string(value);
                    match s.as_str() {
                        "strict" => {
                            config.enforcement = crate::types::EnforcementMode::Strict;
                        }
                        "enforce-new" | "enforce_new" => {
                            config.enforcement = crate::types::EnforcementMode::EnforceNew;
                        }
                        "warn" => {
                            config.enforcement = crate::types::EnforcementMode::Warn;
                        }
                        _ => eprintln!(
                            "Warning: unknown enforcement \"{s}\" (expected \"warn\", \"enforce-new\", or \"strict\")"
                        ),
                    }
                }
                _ => {
                    eprintln!("Warning: unknown key \"{key}\" in config.toml (ignored)");
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

/// Parse a key=value pair inside a `[lifecycle]` TOML section.
fn parse_toml_lifecycle_key(key: &str, value: &str, lc: &mut crate::types::LifecycleConfig) {
    match key {
        "track_history" => lc.track_history = parse_toml_bool(value),
        "allowed_statuses" => lc.allowed_statuses = parse_toml_string_array(value),
        _ => {
            eprintln!("Warning: unknown key \"{key}\" in [lifecycle] section (ignored)");
        }
    }
}

/// Parse a key=value pair inside nested lifecycle sections like `[lifecycle.max_age]`
/// or `[lifecycle.guards."review→active"]`.
fn parse_toml_lifecycle_nested(
    section: &str,
    key: &str,
    value: &str,
    lc: &mut crate::types::LifecycleConfig,
) {
    if section == "lifecycle.max_age" {
        if let Ok(days) = value.trim().parse::<u64>() {
            lc.max_age.insert(key.to_string(), days);
        }
    } else if let Some(guard_name) = section.strip_prefix("lifecycle.guards.") {
        // Strip surrounding quotes from guard name if present
        let name = guard_name.trim_matches('"').to_string();
        let guard = lc.guards.entry(name).or_default();
        match key {
            "min_score" => {
                if let Ok(n) = value.trim().parse::<u32>() {
                    guard.min_score = Some(n);
                }
            }
            "require_sections" => guard.require_sections = parse_toml_string_array(value),
            "no_stale" => guard.no_stale = Some(parse_toml_bool(value)),
            "stale_threshold" => {
                if let Ok(n) = value.trim().parse::<usize>() {
                    guard.stale_threshold = Some(n);
                }
            }
            "message" => guard.message = Some(parse_toml_string(value)),
            _ => {
                eprintln!("Warning: unknown key \"{key}\" in [lifecycle.guards] section (ignored)");
            }
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
    fn test_load_config_toml_takes_priority_over_json() {
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
        // TOML at root takes priority over JSON at root
        assert_eq!(config.specs_dir, "from-toml");
    }

    #[test]
    fn test_load_config_v4_toml_takes_priority() {
        let tmp = TempDir::new().unwrap();
        fs::create_dir_all(tmp.path().join(".specsync")).unwrap();
        fs::write(
            tmp.path().join(".specsync/config.toml"),
            "specs_dir = \"v4-specs\"\nsource_dirs = [\"src\"]\n",
        )
        .unwrap();
        fs::write(
            tmp.path().join("specsync.json"),
            r#"{"specsDir": "legacy", "sourceDirs": ["src"]}"#,
        )
        .unwrap();

        let config = load_config(tmp.path());
        // v4 .specsync/config.toml wins over legacy root files
        assert_eq!(config.specs_dir, "v4-specs");
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
        fs::write(tmp.path().join("specsync.json"), r#"{"specsDir": "specs"}"#).unwrap();

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
        assert_eq!(config.required_sections, vec!["Purpose", "Public API"]);
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
        fs::write(tmp.path().join(".specsync.toml"), "specs_dir = \"specs\"\n").unwrap();

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
