use super::default_schema_pattern;

#[test]
fn test_default_schema_pattern_matches_create_table() {
    let pattern = regex::Regex::new(default_schema_pattern()).unwrap();
    assert!(pattern.is_match("CREATE TABLE users"));
    assert!(pattern.is_match("CREATE TABLE IF NOT EXISTS users"));
    assert!(pattern.is_match("CREATE VIRTUAL TABLE users_fts"));
}

#[test]
fn test_default_schema_pattern_captures_table_name() {
    let pattern = regex::Regex::new(default_schema_pattern()).unwrap();
    let caps = pattern.captures("CREATE TABLE users (id INT)").unwrap();
    assert_eq!(&caps[1], "users");
}
