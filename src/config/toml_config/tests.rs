use super::*;
use crate::config::{load_config, load_config_allowing_unloadable};
use std::fs;
use tempfile::TempDir;

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
    // Invalid TOML (unterminated multi-line array) must fail closed, not
    // silently apply a partial parse of the keys after the damage.
    let tmp = TempDir::new().unwrap();
    fs::write(
            tmp.path().join(".specsync.toml"),
            "source_dirs = [\n  \"src\",\n  \"lib\"\nspecs_dir = \"myspecs\"\nexclude_dirs = [\"target\"]\n",
        )
        .unwrap();
    let config = load_config_allowing_unloadable(tmp.path());
    assert!(
        config
            .load_error
            .as_deref()
            .is_some_and(|error| { error.contains("could not be loaded") }),
        "unterminated TOML array must set load_error, got {:?}",
        config.load_error
    );
}

#[test]
fn test_load_config_toml_unterminated_array_before_section_preserved() {
    // Invalid TOML (unterminated array then a section header) must fail closed
    // rather than absorbing the header into the array and applying defaults.
    let tmp = TempDir::new().unwrap();
    fs::write(
        tmp.path().join(".specsync.toml"),
        "source_dirs = [\n  \"src\"\n[lifecycle]\nallowed_statuses = [\"draft\", \"active\"]\n",
    )
    .unwrap();
    let config = load_config_allowing_unloadable(tmp.path());
    assert!(
        config
            .load_error
            .as_deref()
            .is_some_and(|error| { error.contains("could not be loaded") }),
        "unterminated TOML array before a section must set load_error, got {:?}",
        config.load_error
    );
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
fn test_toml_full_config() {
    let tmp = TempDir::new().unwrap();
    fs::write(
        tmp.path().join(".specsync.toml"),
        r#"
specs_dir = "specs"
source_dirs = ["src", "lib"]
schema_dir = "db/schema"
schema_pattern = 'CREATE TABLE (\w+)'
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
    assert_eq!(
        config.schema_pattern.as_deref(),
        Some(r"CREATE TABLE (\w+)")
    );
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
fn test_toml_invalid_schema_pattern_escape_sets_load_error() {
    // A regex in a TOML basic string cannot use `\\w`; that escape is illegal
    // and must set load_error rather than process::exit via load_config.
    let tmp = TempDir::new().unwrap();
    fs::write(
        tmp.path().join(".specsync.toml"),
        "schema_pattern = \"CREATE TABLE (\\w+)\"\n",
    )
    .unwrap();
    let config = load_config_allowing_unloadable(tmp.path());
    assert!(
        config
            .load_error
            .as_deref()
            .is_some_and(|error| error.contains("could not be loaded")),
        "illegal TOML escape in schema_pattern must set load_error, got {:?}",
        config.load_error
    );
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
fn test_lifecycle_default_track_history_is_true() {
    // The documented default and serde default are both `true`; the Rust
    // `Default` must agree or the TOML reader (which starts from
    // SpecSyncConfig::default()) silently loads an omitted key as false (#10).
    assert!(crate::types::LifecycleConfig::default().track_history);
    assert!(SpecSyncConfig::default().lifecycle.track_history);
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
