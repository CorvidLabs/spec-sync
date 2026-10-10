use super::*;
use crate::config::load_config;
use std::fs;
use tempfile::TempDir;

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
            owns: vec!["tests/core".to_string(), "Package.swift".to_string()],
        },
    );
    config.modules.insert(
        "util".to_string(),
        crate::types::ModuleDefinition {
            files: vec!["src/u.ts".to_string()],
            depends_on: vec![],
            owns: vec![],
        },
    );

    let toml_str = config_to_toml(&config);
    // Deterministic (sorted) output: core before util.
    assert!(
        toml_str.find("[modules.\"core\"]").unwrap() < toml_str.find("[modules.\"util\"]").unwrap(),
        "modules should be emitted in sorted order"
    );

    let reloaded = roundtrip_toml(&config);
    assert_eq!(reloaded.modules.len(), 2);
    let core = reloaded.modules.get("core").expect("core module");
    assert_eq!(core.files, vec!["src/a.ts", "src/b.ts"]);
    assert_eq!(core.depends_on, vec!["util"]);
    assert_eq!(core.owns, vec!["tests/core", "Package.swift"]);
    let util = reloaded.modules.get("util").expect("util module");
    assert_eq!(util.files, vec!["src/u.ts"]);
    assert!(util.depends_on.is_empty());
    assert!(util.owns.is_empty());
}

#[test]
fn test_module_owns_alone_round_trips_and_is_not_a_files_mapping() {
    // A module that owns paths beyond its spec's `files:` and maps no source of its own
    // still carries data, so the writer emits it and the reader materializes it.
    let mut config = SpecSyncConfig::default();
    config.modules.insert(
        "algorand".to_string(),
        crate::types::ModuleDefinition {
            files: vec![],
            depends_on: vec![],
            owns: vec!["Tests/AlgorandTests".to_string()],
        },
    );
    let toml_str = config_to_toml(&config);
    assert!(toml_str.contains("[modules.\"algorand\"]"), "{toml_str}");
    assert!(
        toml_str.contains("owns = [\"Tests/AlgorandTests\"]"),
        "{toml_str}"
    );
    assert!(!toml_str.contains("files ="), "{toml_str}");
    let reloaded = roundtrip_toml(&config);
    let module = reloaded.modules.get("algorand").expect("algorand module");
    assert_eq!(module.owns, vec!["Tests/AlgorandTests"]);
    assert!(module.files.is_empty());
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
            owns: vec![],
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

#[test]
fn config_to_toml_escapes_every_control_character_as_valid_toml() {
    let config = SpecSyncConfig {
        source_dirs: vec!["src\u{0000}\u{0008}\u{000c}\u{0085}".to_string()],
        ..SpecSyncConfig::default()
    };
    let serialized = config_to_toml(&config);
    let parsed: toml::Value =
        toml::from_str(&serialized).expect("serialized config must be valid TOML");
    assert_eq!(
        parsed["source_dirs"][0].as_str(),
        Some("src\u{0000}\u{0008}\u{000c}\u{0085}")
    );
}

/// Write `config_to_toml(config)` to a temp `.specsync.toml`, load it back
/// through the real reader, and return the reloaded config — exercising the
/// full write→read round-trip.
fn roundtrip_toml(config: &SpecSyncConfig) -> SpecSyncConfig {
    let tmp = TempDir::new().unwrap();
    fs::write(tmp.path().join(".specsync.toml"), config_to_toml(config)).unwrap();
    load_config(tmp.path())
}
