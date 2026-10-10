use super::*;
use std::fs;
use tempfile::TempDir;

#[test]
fn test_detect_source_dirs_empty_project() {
    let tmp = TempDir::new().unwrap();
    // Empty dir, no manifest, no source files -> fallback to "src"
    let dirs = detect_source_dirs(tmp.path());
    assert_eq!(dirs, vec!["src"]);
}

#[test]
fn test_detect_source_dirs_with_src_dir() {
    let tmp = TempDir::new().unwrap();
    let src = tmp.path().join("src");
    fs::create_dir(&src).unwrap();
    fs::write(src.join("main.rs"), "fn main() {}").unwrap();

    let dirs = detect_source_dirs(tmp.path());
    assert!(dirs.contains(&"src".to_string()));
}

#[test]
fn test_detect_source_dirs_ignores_node_modules() {
    let tmp = TempDir::new().unwrap();
    let nm = tmp.path().join("node_modules");
    fs::create_dir(&nm).unwrap();
    fs::write(nm.join("index.js"), "module.exports = {}").unwrap();

    let dirs = detect_source_dirs(tmp.path());
    assert!(!dirs.contains(&"node_modules".to_string()));
}

#[test]
fn test_detect_source_dirs_root_source_files() {
    let tmp = TempDir::new().unwrap();
    fs::write(tmp.path().join("main.py"), "print('hello')").unwrap();

    let dirs = detect_source_dirs(tmp.path());
    assert_eq!(dirs, vec!["."]);
}

#[test]
fn test_detect_source_dirs_nested_static_files() {
    let tmp = TempDir::new().unwrap();
    let landing = tmp.path().join("landing");
    fs::create_dir(&landing).unwrap();
    fs::write(landing.join("index.html"), "<main>Welcome</main>").unwrap();

    let dirs = detect_source_dirs(tmp.path());

    assert_eq!(dirs, vec!["landing"]);
}

#[test]
fn test_detect_source_dirs_root_static_files() {
    let tmp = TempDir::new().unwrap();
    fs::write(tmp.path().join("site.css"), "body { color: black; }").unwrap();

    let dirs = detect_source_dirs(tmp.path());

    assert_eq!(dirs, vec!["."]);
}

#[test]
fn checked_source_detection_surfaces_malformed_gradle_settings() {
    let tmp = TempDir::new().unwrap();
    fs::create_dir_all(tmp.path().join("src/main/kotlin")).unwrap();
    fs::write(tmp.path().join("build.gradle.kts"), "plugins {}\n").unwrap();
    fs::write(
        tmp.path().join("settings.gradle.kts"),
        "include(\":member\"\n",
    )
    .unwrap();

    let error = detect_source_dirs_checked(tmp.path()).unwrap_err();
    assert!(error.contains("Cannot parse Gradle settings manifest"));
}
