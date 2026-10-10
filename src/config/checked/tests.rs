use super::*;
use tempfile::TempDir;

#[test]
fn checked_json_config_rejects_wrong_typed_github_repo() {
    let tmp = TempDir::new().unwrap();
    let error = parse_config_content_checked(
        &tmp.path().join("specsync.json"),
        r#"{"specsDir":"specs","sourceDirs":["src"],"github":{"repo":42}}"#,
        tmp.path(),
    )
    .expect_err("retained config parsing must reject a wrong-typed repository");

    assert!(error.contains("github.repo"), "{error}");
}

#[test]
fn checked_json_config_rejects_non_object_github() {
    let tmp = TempDir::new().unwrap();
    let error = parse_config_content_checked(
        &tmp.path().join("specsync.json"),
        r#"{"specsDir":"specs","sourceDirs":["src"],"github":42}"#,
        tmp.path(),
    )
    .expect_err("retained config parsing must reject a non-object GitHub section");

    assert!(error.contains("`github` must be an object"), "{error}");
}

#[test]
fn validate_config_file_rejects_syntax_and_required_field_shapes() {
    let temp = tempfile::TempDir::new().unwrap();
    let toml_path = temp.path().join("config.toml");
    fs::write(
        &toml_path,
        "specs_dir = \"specs\"\nsource_dirs = [\"src\"]\n",
    )
    .unwrap();
    validate_config_file(&toml_path).expect("canonical TOML must validate");

    fs::write(&toml_path, "specs_dir = [\"wrong\"]\n").unwrap();
    let error = validate_config_file(&toml_path).unwrap_err();
    assert!(error.contains("`specs_dir` must be a string"), "{error}");

    let json_path = temp.path().join("config.json");
    fs::write(&json_path, r#"{"specsDir":"specs","sourceDirs":["src"]}"#).unwrap();
    validate_config_file(&json_path).expect("canonical JSON must validate");

    fs::write(&json_path, r#"{"specsDir":"specs","sourceDirs":[1]}"#).unwrap();
    let error = validate_config_file(&json_path).unwrap_err();
    assert!(error.contains("sourceDirs"), "{error}");
}
