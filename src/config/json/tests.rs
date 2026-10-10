use super::*;
use crate::config::load_config;
use std::fs;
use tempfile::TempDir;

#[test]
fn json_github_repo_wrong_types_fail_closed_without_discarding_valid_config() {
    for invalid_repo in [
        serde_json::json!(42),
        serde_json::json!(true),
        serde_json::json!({"owner": "repo"}),
        serde_json::json!(["owner/repo"]),
    ] {
        let tmp = TempDir::new().unwrap();
        let config_path = tmp.path().join("specsync.json");
        let raw = serde_json::json!({
            "specsDir": "configured-specs",
            "sourceDirs": ["configured-src"],
            "github": {
                "repo": invalid_repo,
                "verifyIssues": false
            }
        });
        fs::write(&config_path, serde_json::to_vec(&raw).unwrap()).unwrap();

        let config = load_config(tmp.path());
        assert_eq!(config.specs_dir, "configured-specs");
        assert_eq!(config.source_dirs, ["configured-src"]);
        let github = config
            .github
            .expect("a malformed configured repository must remain explicit");
        assert!(!github.verify_issues);
        assert_eq!(
            github.repo.as_deref(),
            Some(INVALID_JSON_GITHUB_REPO),
            "wrongly typed repositories must not become auto-detection"
        );
        assert!(
            crate::github::resolve_repo(github.repo.as_deref(), tmp.path()).is_err(),
            "wrongly typed repositories must fail before no-reference success"
        );
    }
}

#[test]
fn json_github_repo_absent_null_and_string_remain_compatible() {
    for (github, expected_repo) in [
        (serde_json::json!({}), None),
        (serde_json::json!({"repo": null}), None),
        (
            serde_json::json!({"repo": "CorvidLabs/spec-sync"}),
            Some("CorvidLabs/spec-sync"),
        ),
    ] {
        let tmp = TempDir::new().unwrap();
        let config_path = tmp.path().join("specsync.json");
        let raw = serde_json::json!({
            "sourceDirs": ["src"],
            "github": github
        });
        fs::write(&config_path, serde_json::to_vec(&raw).unwrap()).unwrap();

        let config = load_config(tmp.path());
        assert_eq!(
            config.github.and_then(|github| github.repo).as_deref(),
            expected_repo
        );
    }
}

#[test]
fn test_legacy_json_reads_include_extensionless() {
    let config: SpecSyncConfig =
        serde_json::from_str(r#"{"sourceExtensions": [], "includeExtensionless": true}"#).unwrap();
    assert!(config.source_extensions.is_empty());
    assert!(config.include_extensionless);
}
