use super::*;
use crate::manifest::access::MAX_RETAINED_MANIFEST_ENTRIES;
use crate::manifest::test_support::CountingManifestAccess;
use std::collections::HashSet;
use std::fs;
use std::path::{Path, PathBuf};
use tempfile::tempdir;

#[test]
fn test_parse_cargo_toml_basic() {
    let tmp = tempdir().unwrap();
    fs::create_dir_all(tmp.path().join("src")).unwrap();
    fs::write(tmp.path().join("src/lib.rs"), "").unwrap();
    fs::write(
        tmp.path().join("Cargo.toml"),
        r#"
[package]
name = "my-crate"
version = "0.1.0"

[dependencies]
serde = "1.0"
regex = "1.0"
"#,
    )
    .unwrap();

    let result = parse_cargo_toml(tmp.path()).unwrap();
    assert!(result.modules.contains_key("my-crate"));
    let module = &result.modules["my-crate"];
    assert_eq!(module.source_paths, vec!["src"]);
    assert!(module.dependencies.contains(&"serde".to_string()));
    assert!(module.dependencies.contains(&"regex".to_string()));
}

#[test]
fn cargo_workspace_traversal_charges_declarations_and_memoizes_completed_members() {
    let mut access = CountingManifestAccess::new(MAX_RETAINED_MANIFEST_ENTRIES);
    access.texts.insert(
        PathBuf::from("Cargo.toml"),
        "[workspace]\nmembers = [\"level-one\", \"level-one\"]\n".to_string(),
    );
    access.texts.insert(
        PathBuf::from("level-one/Cargo.toml"),
        "[package]\nname = \"level-one\"\n[workspace]\nmembers = [\"level-two\", \"level-two\"]\n"
            .to_string(),
    );
    access.texts.insert(
        PathBuf::from("level-one/level-two/Cargo.toml"),
        "[package]\nname = \"level-two\"\n".to_string(),
    );

    let mut active = HashSet::new();
    let mut completed = HashSet::new();
    let discovery =
        parse_cargo_toml_with_access(&mut access, Path::new(""), &mut active, &mut completed)
            .unwrap()
            .unwrap();

    assert!(discovery.modules.contains_key("level-one"));
    assert!(discovery.modules.contains_key("level-two"));
    assert_eq!(access.entries, 4);
    assert_eq!(access.text_reads[Path::new("Cargo.toml")], 1);
    assert_eq!(access.text_reads[Path::new("level-one/Cargo.toml")], 1);
    assert_eq!(
        access.text_reads[Path::new("level-one/level-two/Cargo.toml")],
        1
    );
    assert_eq!(completed.len(), 3);
    assert!(active.is_empty());
}

#[test]
fn cargo_workspace_operational_discovery_supports_multiline_members() {
    let mut access = CountingManifestAccess::new(MAX_RETAINED_MANIFEST_ENTRIES);
    access.texts.insert(
        PathBuf::from("Cargo.toml"),
        r#"
[workspace]
members = [
    "crates/core",
    # Inline comments and trailing commas are valid Cargo TOML.
    "crates/cli",
]
"#
        .to_string(),
    );
    access.texts.insert(
        PathBuf::from("crates/core/Cargo.toml"),
        "[package]\nname = \"core\"\nversion = \"0.1.0\"\n".to_string(),
    );
    access.texts.insert(
        PathBuf::from("crates/cli/Cargo.toml"),
        "[package]\nname = \"cli\"\nversion = \"0.1.0\"\n".to_string(),
    );

    let discovery = parse_cargo_toml_with_access(
        &mut access,
        Path::new(""),
        &mut HashSet::new(),
        &mut HashSet::new(),
    )
    .unwrap()
    .unwrap();

    assert!(discovery.modules.contains_key("core"));
    assert!(discovery.modules.contains_key("cli"));
    assert_eq!(access.entries, 2);
    assert_eq!(access.text_reads[Path::new("crates/core/Cargo.toml")], 1);
    assert_eq!(access.text_reads[Path::new("crates/cli/Cargo.toml")], 1);
}

#[test]
fn cargo_workspace_operational_discovery_rejects_malformed_and_wrong_typed_toml() {
    for (content, expected) in [
        ("[workspace\nmembers = []\n", "as TOML"),
        ("workspace = []\n", "`workspace` as a TOML table"),
        (
            "[workspace]\nmembers = \"crates/core\"\n",
            "`workspace.members` as a string array",
        ),
    ] {
        let mut access = CountingManifestAccess::new(MAX_RETAINED_MANIFEST_ENTRIES);
        access
            .texts
            .insert(PathBuf::from("Cargo.toml"), content.to_string());

        let error = parse_cargo_toml_with_access(
            &mut access,
            Path::new(""),
            &mut HashSet::new(),
            &mut HashSet::new(),
        )
        .unwrap_err();

        assert!(error.contains(expected), "{error}");
    }
}

#[test]
fn cargo_workspace_charges_every_member_before_rejecting_a_non_string() {
    let mut access = CountingManifestAccess::new(MAX_RETAINED_MANIFEST_ENTRIES);
    access.texts.insert(
        PathBuf::from("Cargo.toml"),
        "[workspace]\nmembers = [\"crates/core\", 7]\n".to_string(),
    );

    let error = parse_cargo_toml_with_access(
        &mut access,
        Path::new(""),
        &mut HashSet::new(),
        &mut HashSet::new(),
    )
    .unwrap_err();

    assert!(error.contains("non-string workspace member"), "{error}");
    assert_eq!(access.entries, 2);
}
