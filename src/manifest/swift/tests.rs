use super::*;
use std::fs;
use tempfile::tempdir;

#[test]
fn test_parse_package_swift_basic() {
    let tmp = tempdir().unwrap();
    fs::create_dir_all(tmp.path().join("Sources/MyLib")).unwrap();
    fs::write(
        tmp.path().join("Package.swift"),
        r#"
// swift-tools-version:5.9
import PackageDescription

let package = Package(
    name: "MyPackage",
    targets: [
        .target(name: "MyLib", dependencies: ["Logging"]),
        .target(name: "MyApp", dependencies: [.target(name: "MyLib")], path: "Sources/App"),
        .testTarget(name: "MyLibTests", dependencies: ["MyLib"]),
    ]
)
"#,
    )
    .unwrap();

    let result = parse_package_swift(tmp.path()).unwrap();
    assert!(result.modules.contains_key("MyLib"));
    assert!(result.modules.contains_key("MyApp"));
    // testTarget should NOT be in modules
    assert!(!result.modules.contains_key("MyLibTests"));

    let mylib = &result.modules["MyLib"];
    assert_eq!(mylib.source_paths, vec!["Sources/MyLib"]);
    assert!(mylib.dependencies.contains(&"Logging".to_string()));

    let myapp = &result.modules["MyApp"];
    assert_eq!(myapp.source_paths, vec!["Sources/App"]);
}

#[test]
fn test_extract_balanced_parens() {
    assert_eq!(
        extract_balanced_parens("name: \"Foo\", path: \"bar\")"),
        Some("name: \"Foo\", path: \"bar\"".to_string())
    );
    assert_eq!(
        extract_balanced_parens("a(b), c)"),
        Some("a(b), c".to_string())
    );
    assert_eq!(extract_balanced_parens("no close paren"), None);
}
