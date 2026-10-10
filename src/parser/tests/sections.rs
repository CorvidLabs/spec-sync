use super::*;

#[test]
fn test_scaffold_boilerplate_detected() {
    // Regression (#441): an untouched `specsync new` scaffold must not
    // count as placeholder-free, meaningful content.
    assert!(is_boilerplate_line(
        "Document this module's responsibility, inputs, outputs, and ownership boundaries."
    ));
    assert!(is_boilerplate_line("- **Given** precondition"));
    assert!(is_boilerplate_line(
        "1. Define an invariant that must remain true for supported inputs."
    ));
    assert!(!is_boilerplate_line("Handles authentication tokens."));
}

#[test]
fn test_get_missing_sections() {
    let body = "## Purpose\nSomething\n\n## Public API\nStuff\n";
    let required = vec![
        "Purpose".to_string(),
        "Public API".to_string(),
        "Invariants".to_string(),
    ];
    let missing = get_missing_sections(body, &required);
    assert_eq!(missing, vec!["Invariants"]);
}

#[test]
fn test_body_has_section_exact_match() {
    let body = "## Purpose\nSome text\n\n## PurposeFoo\nOther text\n";
    assert!(body_has_section(body, "Purpose"));
    assert!(body_has_section(body, "PurposeFoo"));
    // "Purpose" should NOT match "PurposeFoo" and vice versa
    assert!(!body_has_section(body, "PurposeF"));
}

#[test]
fn test_body_has_section_trailing_whitespace() {
    // Header with trailing whitespace in body should still match
    let body = "## Purpose   \nSome text\n";
    assert!(body_has_section(body, "Purpose"));
}

#[test]
fn test_get_missing_sections_no_false_positives() {
    // "## Purpose" should not satisfy a requirement for "## PurposeFoo"
    let body = "## Purpose\nContent\n\n## Design\nContent\n";
    let missing = get_missing_sections(body, &["PurposeFoo".to_string()]);
    assert_eq!(missing, vec!["PurposeFoo"]);
    // But "Purpose" itself is present
    let missing = get_missing_sections(body, &["Purpose".to_string()]);
    assert!(missing.is_empty());
}

#[test]
fn test_section_has_content_no_false_positives() {
    // "## Purpose" should not be used to satisfy "## PurposeFoo"
    let body = "## Purpose\nReal content here.\n\n## PurposeFoo\n\n";
    assert!(section_has_content(body, "Purpose"));
    assert!(!section_has_content(body, "PurposeFoo"));
}

#[test]
fn test_section_has_content_real() {
    let body = "## Purpose\nThis module handles authentication.\n\n## Invariants\n1. Tokens must be valid\n";
    assert!(section_has_content(body, "Purpose"));
    assert!(section_has_content(body, "Invariants"));
}

#[test]
fn test_section_has_content_empty() {
    let body = "## Purpose\n\n## Invariants\n";
    assert!(!section_has_content(body, "Purpose"));
}

#[test]
fn test_section_has_content_stub_tbd() {
    let body = "## Purpose\nTBD\n\n## Invariants\n- N/A\n";
    assert!(!section_has_content(body, "Purpose"));
    assert!(!section_has_content(body, "Invariants"));
}

#[test]
fn test_section_has_content_stub_phrases() {
    let body =
        "## Purpose\nTo be determined\n\n## Error Cases\nComing soon\n\n## Dependencies\nTBD\n";
    assert!(!section_has_content(body, "Purpose"));
    assert!(!section_has_content(body, "Error Cases"));
    assert!(!section_has_content(body, "Dependencies"));
}

#[test]
fn test_section_has_content_none_is_valid() {
    // "None." is legitimate content (e.g. "no dependencies")
    let body = "## Dependencies\nNone.\n";
    assert!(section_has_content(body, "Dependencies"));
}

#[test]
fn test_section_has_content_table_header_only() {
    let body =
        "## Public API\n\n| Export | Description |\n|--------|-------------|\n\n## Invariants\n";
    assert!(!section_has_content(body, "Public API"));
}

#[test]
fn test_section_has_content_table_with_data() {
    let body = "## Public API\n\n| Export | Description |\n|--------|-------------|\n| `foo` | Does things |\n\n## Invariants\n";
    assert!(section_has_content(body, "Public API"));
}

#[test]
fn test_find_stub_sections() {
    let body = "## Purpose\nReal content here\n\n## Public API\nTBD\n\n## Invariants\nN/A\n\n## Error Cases\n| Condition | Behavior |\n|-----------|----------|\n| Bad input | Returns error |\n";
    let required = vec![
        "Purpose".to_string(),
        "Public API".to_string(),
        "Invariants".to_string(),
        "Error Cases".to_string(),
    ];
    let stubs = find_stub_sections(body, &required);
    assert_eq!(stubs, vec!["Public API", "Invariants"]);
}

#[test]
fn test_find_stub_sections_none() {
    let body = "## Purpose\nReal content\n\n## Public API\n| Export | Desc |\n|--------|------|\n| `foo` | Bar |\n";
    let required = vec!["Purpose".to_string(), "Public API".to_string()];
    let stubs = find_stub_sections(body, &required);
    assert!(stubs.is_empty());
}

#[test]
fn generator_placeholder_text_counts_as_stub() {
    // #421: the generators' own scaffold sentences are draft markers, not
    // real content — a section filled only with them is a stub.
    let body = "## Purpose\n\nDocument this module's responsibility, inputs, outputs, and ownership boundaries.\n\n## Invariants\n\n1. Define an invariant that must remain true for supported inputs.\n";
    assert!(!section_has_content(body, "Purpose"));
    assert!(!section_has_content(body, "Invariants"));

    let behavioral = "## Behavioral Examples\n\n### Scenario: Core behavior\n\n- **Given** precondition\n- **When** action\n- **Then** result\n";
    assert!(!section_has_content(behavioral, "Behavioral Examples"));

    // ...but real content is still real content.
    let real = "## Purpose\n\nAuthenticates users via OAuth2 and session tokens.\n";
    assert!(section_has_content(real, "Purpose"));
}
