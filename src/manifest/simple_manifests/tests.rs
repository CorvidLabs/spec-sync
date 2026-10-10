use super::*;
use std::fs;
use tempfile::tempdir;

#[test]
fn test_parse_go_mod() {
    let tmp = tempdir().unwrap();
    fs::create_dir_all(tmp.path().join("cmd")).unwrap();
    fs::create_dir_all(tmp.path().join("internal")).unwrap();
    fs::write(
        tmp.path().join("go.mod"),
        "module github.com/user/myproject\n\ngo 1.21\n",
    )
    .unwrap();

    let result = parse_go_mod(tmp.path()).unwrap();
    assert!(result.modules.contains_key("myproject"));
    assert!(result.source_dirs.contains(&"cmd".to_string()));
    assert!(result.source_dirs.contains(&"internal".to_string()));
}
