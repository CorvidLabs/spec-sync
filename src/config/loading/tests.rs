use super::*;
use tempfile::TempDir;

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
fn test_load_config_malformed_json_sets_load_error() {
    let tmp = TempDir::new().unwrap();
    fs::write(tmp.path().join("specsync.json"), "not valid json {{{").unwrap();

    let config = load_config_allowing_unloadable(tmp.path());
    assert!(
        config.load_error.as_deref().is_some_and(|error| {
            error.contains("could not be loaded") && error.contains("specsync.json")
        }),
        "malformed JSON must set load_error, got {:?}",
        config.load_error
    );
    assert_eq!(config.specs_dir, "specs"); // default standing in, not applied
}

#[test]
fn test_load_config_malformed_v4_json_sets_load_error() {
    let tmp = TempDir::new().unwrap();
    fs::create_dir_all(tmp.path().join(".specsync")).unwrap();
    fs::write(
        tmp.path().join(".specsync/config.json"),
        "{ not json at all",
    )
    .unwrap();

    let config = load_config_allowing_unloadable(tmp.path());
    assert!(
        config.load_error.as_deref().is_some_and(|error| {
            error.contains("could not be loaded") && error.contains("config.json")
        }),
        "malformed v4 JSON must set load_error, got {:?}",
        config.load_error
    );
}

#[test]
fn test_load_config_malformed_toml_sets_load_error() {
    let tmp = TempDir::new().unwrap();
    fs::create_dir_all(tmp.path().join(".specsync")).unwrap();
    fs::write(
        tmp.path().join(".specsync/config.toml"),
        "not = [ valid toml {{{",
    )
    .unwrap();

    let config = load_config_allowing_unloadable(tmp.path());
    assert!(
        config.load_error.as_deref().is_some_and(|error| {
            error.contains("could not be loaded") && error.contains("config.toml")
        }),
        "malformed TOML must set load_error, got {:?}",
        config.load_error
    );
    assert_eq!(config.specs_dir, "specs");
}

#[test]
fn test_load_config_empty_toml_sets_load_error() {
    let tmp = TempDir::new().unwrap();
    fs::create_dir_all(tmp.path().join(".specsync")).unwrap();
    fs::write(tmp.path().join(".specsync/config.toml"), "   \n").unwrap();

    let config = load_config_allowing_unloadable(tmp.path());
    assert!(
        config.load_error.as_deref().is_some_and(|error| {
            error.contains("could not be loaded") && error.contains("config.toml")
        }),
        "empty TOML must set load_error, got {:?}",
        config.load_error
    );
}

#[test]
fn test_load_config_directory_as_toml_sets_load_error() {
    let tmp = TempDir::new().unwrap();
    fs::create_dir_all(tmp.path().join(".specsync/config.toml")).unwrap();

    let config = load_config_allowing_unloadable(tmp.path());
    assert!(
        config.load_error.as_deref().is_some_and(|error| {
            error.contains("could not be loaded") && error.contains("config.toml")
        }),
        "directory standing in for config.toml must set load_error, got {:?}",
        config.load_error
    );
}

#[test]
fn test_load_config_invalid_enforcement_enum_sets_load_error() {
    let tmp = TempDir::new().unwrap();
    fs::create_dir_all(tmp.path().join(".specsync")).unwrap();
    fs::write(
        tmp.path().join(".specsync/config.toml"),
        "enforcement = \"nope\"\n",
    )
    .unwrap();

    let config = load_config_allowing_unloadable(tmp.path());
    assert!(
        config.load_error.as_deref().is_some_and(|error| {
            error.contains("could not be loaded") && error.contains("config.toml")
        }),
        "invalid enforcement enum must set load_error, got {:?}",
        config.load_error
    );
}

#[test]
fn test_load_config_json_without_source_dirs_auto_detects() {
    let tmp = TempDir::new().unwrap();
    fs::write(tmp.path().join("specsync.json"), r#"{"specsDir": "specs"}"#).unwrap();

    let config = load_config(tmp.path());
    // sourceDirs not in JSON, so it should auto-detect
    assert!(!config.source_dirs.is_empty());
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
