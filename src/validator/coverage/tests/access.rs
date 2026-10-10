use super::super::{MAX_COVERAGE_FILE_BYTES, MAX_COVERAGE_INPUT_BYTES};
use super::*;
use std::fs;
use std::path::Path;

#[test]
fn retained_coverage_file_read_rejects_preopen_regular_replacement() {
    let project = tempfile::tempdir().unwrap();
    let source = project.path().join("source.rs");
    let replacement = project.path().join("replacement.rs");
    fs::write(&source, "pub fn original() {}\n").unwrap();
    fs::write(&replacement, "pub fn replacement() {}\n").unwrap();
    let retained = open_coverage_project_root(project.path()).unwrap();

    let error = read_coverage_file_with_hook(
        &retained,
        std::ffi::OsStr::new("source.rs"),
        Path::new("source.rs"),
        MAX_COVERAGE_FILE_BYTES,
        MAX_COVERAGE_INPUT_BYTES,
        MAX_COVERAGE_INPUT_BYTES,
        || {
            fs::remove_file(&source).unwrap();
            fs::rename(&replacement, &source).unwrap();
        },
    )
    .unwrap_err();

    assert!(error.contains("changed during retained open"), "{error}");
}
