use super::*;
use crate::manifest::access::{MAX_RETAINED_MANIFEST_ENTRIES, display_manifest_path};
use crate::manifest::test_support::CountingManifestAccess;
use std::fs;
use std::path::{Path, PathBuf};
use tempfile::tempdir;

#[test]
fn test_parse_package_json_workspaces() {
    let tmp = tempdir().unwrap();
    fs::create_dir_all(tmp.path().join("packages/core/src")).unwrap();
    fs::create_dir_all(tmp.path().join("packages/web/src")).unwrap();
    fs::write(
        tmp.path().join("packages/core/package.json"),
        r#"{"name": "@app/core"}"#,
    )
    .unwrap();
    fs::write(
        tmp.path().join("packages/web/package.json"),
        r#"{"name": "@app/web"}"#,
    )
    .unwrap();
    fs::create_dir_all(tmp.path().join("src")).unwrap();
    fs::write(
        tmp.path().join("package.json"),
        r#"{"name": "my-app", "workspaces": ["packages/*"]}"#,
    )
    .unwrap();

    let result = parse_package_json(tmp.path()).unwrap();
    assert!(result.modules.contains_key("core"));
    assert!(result.modules.contains_key("web"));
    assert!(
        result
            .source_dirs
            .contains(&"packages/core/src".to_string())
    );
}

#[test]
fn node_workspace_traversal_charges_patterns_and_deduplicates_cached_expansion() {
    let mut access = CountingManifestAccess::new(MAX_RETAINED_MANIFEST_ENTRIES);
    access.texts.insert(
        PathBuf::from("package.json"),
        r#"{"name":"root","workspaces":["packages/*","packages/**"]}"#.to_string(),
    );
    access
        .children
        .insert(PathBuf::from("packages"), vec!["member".to_string()]);
    access.texts.insert(
        PathBuf::from("packages/member/package.json"),
        r#"{"name":"member"}"#.to_string(),
    );
    access
        .directories
        .insert(PathBuf::from("packages/member/src"));

    let discovery = parse_package_json_with_access(&mut access)
        .unwrap()
        .unwrap();

    assert!(discovery.modules.contains_key("member"));
    assert_eq!(access.entries, 3);
    assert_eq!(access.child_reads[Path::new("packages")], 1);
    assert_eq!(
        access.text_reads[Path::new("packages/member/package.json")],
        1
    );
}

#[test]
fn node_workspace_object_form_preserves_yarn_and_pnpm_package_metadata() {
    let mut access = CountingManifestAccess::new(MAX_RETAINED_MANIFEST_ENTRIES);
    access.texts.insert(
        PathBuf::from("package.json"),
        r#"{
                "name": "root",
                "packageManager": "pnpm@9.0.0",
                "workspaces": {
                    "packages": ["packages/*"],
                    "nohoist": ["**/native"]
                }
            }"#
        .to_string(),
    );
    access
        .children
        .insert(PathBuf::from("packages"), vec!["member".to_string()]);
    access.texts.insert(
        PathBuf::from("packages/member/package.json"),
        r#"{"name":"member"}"#.to_string(),
    );

    let discovery = parse_package_json_with_access(&mut access)
        .unwrap()
        .unwrap();

    assert!(discovery.modules.contains_key("member"));
    assert_eq!(access.entries, 2);
}

#[test]
fn node_package_manifests_fail_closed_for_malformed_and_wrong_typed_inputs() {
    for (content, expected) in [
        ("{\"name\":", "as JSON"),
        ("[]", "must contain a JSON object"),
        (
            r#"{"workspaces":"packages/*"}"#,
            "`workspaces` as a string array",
        ),
        (
            r#"{"workspaces":{"packages":"packages/*"}}"#,
            "`workspaces.packages` as a string array",
        ),
        (
            r#"{"workspaces":{"nohoist":[]}}"#,
            "`workspaces.packages` as a string array",
        ),
    ] {
        let mut access = CountingManifestAccess::new(MAX_RETAINED_MANIFEST_ENTRIES);
        access
            .texts
            .insert(PathBuf::from("package.json"), content.to_string());

        let error = parse_package_json_with_access(&mut access).unwrap_err();

        assert!(error.contains(expected), "{error}");
    }
}

#[test]
fn node_workspace_charges_every_entry_before_rejecting_a_non_string() {
    for content in [
        r#"{"workspaces":["packages/*",7]}"#,
        r#"{"workspaces":{"packages":["packages/*",7]}}"#,
    ] {
        let mut access = CountingManifestAccess::new(MAX_RETAINED_MANIFEST_ENTRIES);
        access
            .texts
            .insert(PathBuf::from("package.json"), content.to_string());

        let error = parse_package_json_with_access(&mut access).unwrap_err();

        assert!(error.contains("non-string workspace entry"), "{error}");
        assert_eq!(access.entries, 2);
    }
}

#[test]
fn nested_node_package_manifest_is_strict_json_object_input() {
    let mut access = CountingManifestAccess::new(MAX_RETAINED_MANIFEST_ENTRIES);
    access.texts.insert(
        PathBuf::from("package.json"),
        r#"{"workspaces":["packages/*"]}"#.to_string(),
    );
    access
        .children
        .insert(PathBuf::from("packages"), vec!["member".to_string()]);
    access.texts.insert(
        PathBuf::from("packages/member/package.json"),
        "[]".to_string(),
    );

    let error = parse_package_json_with_access(&mut access).unwrap_err();

    let manifest_path = Path::new("packages").join("member").join("package.json");
    assert!(
        error.contains(&display_manifest_path(&manifest_path)),
        "{error}"
    );
    assert!(error.contains("must contain a JSON object"), "{error}");
}
