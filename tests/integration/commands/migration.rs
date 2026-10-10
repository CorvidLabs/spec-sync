//! Legacy project migration, recovery, and idempotence.

use crate::helpers::*;
use predicates::prelude::*;
use std::fs;
use tempfile::TempDir;

// ─── specsync migrate ──────────────────────────────────────────────────

#[test]
fn migrate_full_v3_to_v4() {
    let tmp = TempDir::new().unwrap();
    let root = tmp.path().to_path_buf();
    setup_v3_project(&root);

    // Run migration
    specsync()
        .args(["migrate", "--root"])
        .arg(&root)
        .assert()
        .success()
        .stdout(predicate::str::contains("Successfully migrated to v4.0.0"));

    // Verify directory structure
    assert!(root.join(".specsync").exists(), ".specsync/ should exist");
    assert!(
        root.join(".specsync/lifecycle").exists(),
        ".specsync/lifecycle/ should exist"
    );
    assert!(
        root.join(".specsync/changes").exists(),
        ".specsync/changes/ should exist"
    );
    assert!(
        root.join(".specsync/archive").exists(),
        ".specsync/archive/ should exist"
    );

    // Config relocated
    assert!(
        root.join(".specsync/config.toml").exists(),
        "config.toml should exist"
    );
    assert!(
        !root.join("specsync.json").exists(),
        "specsync.json should be removed"
    );

    // Registry relocated
    assert!(
        root.join(".specsync/registry.toml").exists(),
        "registry.toml should exist"
    );
    assert!(
        !root.join("specsync-registry.toml").exists(),
        "specsync-registry.toml should be removed"
    );

    // Lifecycle extracted
    assert!(
        root.join(".specsync/lifecycle/auth.json").exists(),
        "lifecycle/auth.json should exist"
    );

    // Lifecycle log removed from spec frontmatter
    let spec_content = fs::read_to_string(root.join("specs/auth/auth.spec.md")).unwrap();
    assert!(
        !spec_content.contains("lifecycle_log:"),
        "lifecycle_log should be removed from spec"
    );

    // Version stamped
    let version = fs::read_to_string(root.join(".specsync/version")).unwrap();
    assert_eq!(version.trim(), "4.0.0");

    // Backup created
    assert!(
        root.join(".specsync/backup-3x/manifest.json").exists(),
        "backup manifest should exist"
    );
    assert!(
        root.join(".specsync/backup-3x/specsync.json").exists(),
        "backup of specsync.json should exist"
    );

    // Gitignore created
    assert!(
        root.join(".specsync/.gitignore").exists(),
        ".gitignore should exist"
    );
    let gitignore = fs::read_to_string(root.join(".specsync/.gitignore")).unwrap();
    assert!(
        gitignore.contains("backup-3x/"),
        "gitignore should ignore backup-3x"
    );
    // archive/ should not be gitignored (part of the v4 lifecycle)
    let archive_is_ignored = gitignore
        .lines()
        .any(|line| !line.starts_with('#') && line.trim() == "archive/");
    assert!(!archive_is_ignored, "gitignore should NOT ignore archive");
    // hashes.json SHOULD be gitignored (local-only cache, regenerated on each run)
    let hashes_is_ignored = gitignore
        .lines()
        .any(|line| !line.starts_with('#') && line.trim() == "hashes.json");
    assert!(hashes_is_ignored, "gitignore SHOULD ignore hashes.json");
    // Also check root .gitignore has .specsync/hashes.json
    let root_gitignore = fs::read_to_string(root.join(".gitignore")).unwrap_or_default();
    assert!(
        root_gitignore.contains(".specsync/hashes.json"),
        "root .gitignore should contain .specsync/hashes.json"
    );
}

#[test]
fn migrate_check_passes_after_migration() {
    let tmp = TempDir::new().unwrap();
    let root = tmp.path().to_path_buf();
    setup_v3_project(&root);

    // Migrate
    specsync()
        .args(["migrate", "--root"])
        .arg(&root)
        .assert()
        .success();

    // Check should pass on the migrated project
    specsync()
        .args(["check", "--root"])
        .arg(&root)
        .assert()
        .success()
        .stdout(predicate::str::contains("0 failed"));
}

#[test]
fn migrate_idempotent_rerun_is_noop() {
    let tmp = TempDir::new().unwrap();
    let root = tmp.path().to_path_buf();
    setup_v3_project(&root);

    // First migration
    specsync()
        .args(["migrate", "--root"])
        .arg(&root)
        .assert()
        .success()
        .stdout(predicate::str::contains("Successfully migrated"));

    // Second migration should be a no-op
    specsync()
        .args(["migrate", "--root"])
        .arg(&root)
        .assert()
        .success()
        .stdout(predicate::str::contains("Already at v4.0.0"));
}

#[test]
fn migrate_dry_run_no_side_effects() {
    let tmp = TempDir::new().unwrap();
    let root = tmp.path().to_path_buf();
    setup_v3_project(&root);

    // Dry run
    specsync()
        .args(["migrate", "--dry-run", "--root"])
        .arg(&root)
        .assert()
        .success()
        .stdout(predicate::str::contains("Dry run complete"));

    // Nothing should have changed
    assert!(
        root.join("specsync.json").exists(),
        "specsync.json should still exist after dry-run"
    );
    assert!(
        root.join("specsync-registry.toml").exists(),
        "registry should still exist after dry-run"
    );
    assert!(
        !root.join(".specsync/config.toml").exists(),
        "config.toml should NOT exist after dry-run"
    );
    assert!(
        !root.join(".specsync/version").exists(),
        "version file should NOT exist after dry-run"
    );

    // Spec should still have lifecycle_log
    let spec_content = fs::read_to_string(root.join("specs/auth/auth.spec.md")).unwrap();
    assert!(
        spec_content.contains("lifecycle_log:"),
        "lifecycle_log should still be in spec after dry-run"
    );
}

#[test]
fn migrate_json_output_format() {
    let tmp = TempDir::new().unwrap();
    let root = tmp.path().to_path_buf();
    setup_v3_project(&root);

    let output = specsync()
        .args(["migrate", "--format", "json", "--root"])
        .arg(&root)
        .output()
        .unwrap();

    assert!(output.status.success());

    let stdout = String::from_utf8(output.stdout).unwrap();
    let json: serde_json::Value = serde_json::from_str(&stdout).unwrap();
    assert_eq!(json["status"], "completed");
    assert_eq!(json["version"], "4.0.0");
    assert_eq!(json["dry_run"], false);
}

#[test]
fn migrate_no_project_fails() {
    let tmp = TempDir::new().unwrap();
    let root = tmp.path().to_path_buf();

    // Empty directory — no spec-sync project
    specsync()
        .args(["migrate", "--root"])
        .arg(&root)
        .assert()
        .failure();
}

#[test]
fn migrate_preserves_unparseable_config_and_fails() {
    // Regression: a single parse error (here a trailing comma) must not cause
    // migrate to write a pure-default config.toml, delete the original, and
    // report success. It must fail loudly and leave the project untouched.
    let tmp = TempDir::new().unwrap();
    let root = tmp.path().to_path_buf();
    let original = "{\n  \"sourceDirs\": [\"lib\"],\n  \"enforcement\": \"strict\",\n}\n";
    fs::write(root.join("specsync.json"), original).unwrap();
    fs::create_dir_all(root.join("specs")).unwrap();

    specsync()
        .args(["migrate", "--no-backup", "--root"])
        .arg(&root)
        .assert()
        .failure();

    // Original config preserved byte-for-byte; no default config written.
    assert_eq!(
        fs::read_to_string(root.join("specsync.json")).unwrap(),
        original,
        "the original (malformed) config must be left untouched"
    );
    assert!(
        !root.join(".specsync/config.toml").exists(),
        "no default config.toml should have been written"
    );
    // And no version stamp that would make a re-run refuse to migrate.
    assert!(
        !root.join(".specsync/version").exists(),
        "migration must not stamp a version when it aborted"
    );
}

#[test]
fn migrate_no_backup_flag() {
    let tmp = TempDir::new().unwrap();
    let root = tmp.path().to_path_buf();
    setup_v3_project(&root);

    specsync()
        .args(["migrate", "--no-backup", "--root"])
        .arg(&root)
        .assert()
        .success()
        .stdout(predicate::str::contains("Successfully migrated"));

    // Backup should NOT exist
    assert!(
        !root.join(".specsync/backup-3x/manifest.json").exists(),
        "backup should not exist with --no-backup"
    );

    // But migration should still be complete
    let version = fs::read_to_string(root.join(".specsync/version")).unwrap();
    assert_eq!(version.trim(), "4.0.0");
}

#[test]
fn migrate_partial_recovery() {
    let tmp = TempDir::new().unwrap();
    let root = tmp.path().to_path_buf();
    setup_v3_project(&root);

    // Simulate a partial migration: create .specsync/ with version but leave old config
    fs::create_dir_all(root.join(".specsync/lifecycle")).unwrap();
    fs::create_dir_all(root.join(".specsync/changes")).unwrap();
    fs::create_dir_all(root.join(".specsync/archive")).unwrap();
    // Don't write version file — so migrate should detect partial state and continue

    // Run migrate — should complete the remaining steps
    specsync()
        .args(["migrate", "--root"])
        .arg(&root)
        .assert()
        .success()
        .stdout(predicate::str::contains("Successfully migrated"));

    // Verify full migration completed
    assert!(root.join(".specsync/config.toml").exists());
    assert!(root.join(".specsync/version").exists());
    let version = fs::read_to_string(root.join(".specsync/version")).unwrap();
    assert_eq!(version.trim(), "4.0.0");
}
