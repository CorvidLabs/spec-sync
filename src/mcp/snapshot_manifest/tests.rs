use super::*;

#[test]
fn snapshot_manifest_input_rejects_true_root_and_cross_platform_lexical_escapes() {
    let error = snapshot_manifest_input(
        Path::new("crates/a"),
        "../../../outside",
        "Cargo target path",
    )
    .expect_err("parent traversal beyond the retained root must be rejected");
    assert!(error.contains("escapes the configured server root"));

    for configured in [
        "/outside",
        "C:/outside",
        "C:outside",
        r"\\server\share\outside",
        r"\outside",
    ] {
        let error = snapshot_manifest_input(Path::new("crates/a"), configured, "Cargo target path")
            .expect_err("absolute, drive, UNC, and backslash paths must be rejected portably");
        assert!(error.contains("safe project-relative path"));
    }

    let error = snapshot_manifest_input(
        Path::new("crates/a"),
        r"..\..\..\outside",
        "Cargo target path",
    )
    .expect_err("Windows-native traversal beyond the retained root must be rejected");
    assert!(error.contains("escapes the configured server root"));
}
