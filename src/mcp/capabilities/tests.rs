use super::*;
use crate::mcp::tests::open_test_directory;
use std::fs;
use tempfile::TempDir;

#[cfg(unix)]
#[test]
fn server_root_capability_rejects_a_root_replaced_before_canonicalization() {
    use std::os::unix::fs::symlink;

    let tmp = TempDir::new().unwrap();
    let root = tmp.path().join("server");
    let moved = tmp.path().join("moved-server");
    let outside = tmp.path().join("outside");
    fs::create_dir_all(&root).unwrap();
    fs::create_dir_all(&outside).unwrap();

    let error = open_server_root_capability(&root, || {
        fs::rename(&root, &moved).unwrap();
        symlink(&outside, &root).unwrap();
        Ok(())
    })
    .expect_err("a swapped root must be rejected");

    assert!(error.contains("root") || error.contains("capability"));
}

#[test]
fn selected_config_directory_identity_stays_bound_to_the_pre_open_observation() {
    assert!(snapshot_configuration_identities_match(
        &1_u64,
        &[Some(1), Some(1), Some(1)]
    ));
    assert!(!snapshot_configuration_identities_match(
        &1_u64,
        &[Some(2), Some(2), Some(2)]
    ));
    assert!(!snapshot_configuration_identities_match(
        &1_u64,
        &[Some(1), None, Some(1)]
    ));
}

#[test]
fn selected_config_parent_open_binds_the_pre_open_identity() {
    let tmp = TempDir::new().unwrap();
    let root = tmp.path();
    fs::create_dir(root.join("selected")).unwrap();
    fs::create_dir(root.join("replacement")).unwrap();
    let source = open_test_directory(root);

    let result = open_snapshot_configuration_directory_with_hook(
        &source,
        OsStr::new("selected"),
        Path::new("selected"),
        || {
            fs::rename(root.join("selected"), root.join("original")).unwrap();
            fs::rename(root.join("replacement"), root.join("selected")).unwrap();
        },
    );

    let error =
        result.expect_err("a replacement opened after metadata inspection must be rejected");
    assert!(error.contains("changed during inspection"), "{error}");
}

#[test]
fn file_identity_rejects_changed_bytes_on_the_same_filesystem_entry() {
    let tmp = TempDir::new().unwrap();
    fs::write(tmp.path().join("generated.spec.md"), "generated\n").unwrap();
    let parent = open_test_directory(tmp.path());
    let original = file_identity(&parent.open("generated.spec.md").unwrap()).unwrap();

    fs::write(tmp.path().join("generated.spec.md"), "replacement\n").unwrap();
    let replacement = file_identity(&parent.open("generated.spec.md").unwrap()).unwrap();

    assert_ne!(original, replacement);
}

#[test]
fn file_identity_digest_fails_closed_at_the_generated_output_bound() {
    let tmp = TempDir::new().unwrap();
    fs::write(tmp.path().join("oversized"), b"123456789").unwrap();
    let parent = open_test_directory(tmp.path());
    let file = parent.open("oversized").unwrap();

    let error = file_content_digest_with_limit(&file, 8).unwrap_err();

    assert_eq!(error.kind(), io::ErrorKind::InvalidData);
    assert!(error.to_string().contains("8-byte limit"));
}
