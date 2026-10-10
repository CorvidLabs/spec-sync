use super::*;

#[test]
fn test_parse_checked_issue_references_uses_the_one_delimiter_rule() {
    assert_eq!(
        parse_checked_issue_references("---  \nimplements: [7]\n---\n\nBody.\n").unwrap(),
        (vec![7], vec![]),
        "a padded opener"
    );
    assert_eq!(
        parse_checked_issue_references("---\nimplements: [7]\n---  \n\nBody.\n\n---\n\nMore.\n")
            .unwrap(),
        (vec![7], vec![]),
        "a padded closer, with a horizontal rule below it in the body"
    );
    assert_eq!(
        parse_checked_issue_references("---\nimplements: [7]\r\n---\r\n\r\nBody.\r\n").unwrap(),
        (vec![7], vec![]),
        "delimiters need not agree with each other about line endings"
    );
}

// Honest label: CONTROL, and it passes on the unfixed binary too. Routing this reader through
// the shared scan must not change WHICH documents it accepts beyond the delimiter shape.
// An empty block was refused before — `split_once` could not produce one — and a block that
// is a single blank line was accepted; both verdicts are inherited deliberately, so a later
// "tidy-up" that collapses them has to argue for it.
#[test]
fn test_parse_checked_issue_references_keeps_its_empty_block_verdicts() {
    assert_eq!(
        parse_checked_issue_references("---\n---\n\nBody.\n").unwrap_err(),
        "missing or malformed YAML frontmatter"
    );
    assert_eq!(
        parse_checked_issue_references("---\n\n---\n\nBody.\n").unwrap(),
        (vec![], vec![])
    );
    assert_eq!(
        parse_checked_issue_references("# Notes\n\nNo frontmatter here.\n").unwrap_err(),
        "missing or malformed YAML frontmatter"
    );
}

// Honest label: DISCRIMINATOR, and the guard against this repository's most-repeated failure
// — a fix landing at the site the report names while a parallel implementation survives.
// Three readers in this module decide what a delimiter is; the rule is written twice, once as
// `is_frontmatter_delimiter` and once inside `FRONTMATTER_RE`. This is what fails if they
// ever drift.
//
// Discriminates: on the unfixed binary every padded row disagrees — `strip_frontmatter`
// returns the document whole while the others error, or `strip_frontmatter` deletes body
// prose while `parse_frontmatter` merely truncates it.
#[test]
fn checked_issue_references_accept_normal_lists_and_inline_comments() {
    let content = "\
---
module: auth
implements:
  - 41
  - 42 # inline item comment
tracks: [43, 44] # inline list comment
---

# Auth
";

    assert_eq!(
        parse_checked_issue_references(content).unwrap(),
        (vec![41, 42], vec![43, 44])
    );

    let max = "---\nimplements: [18446744073709551615]\n---\n\n# Auth\n";
    assert_eq!(
        parse_checked_issue_references(max).unwrap(),
        (vec![u64::MAX], Vec::new())
    );
}

#[test]
fn checked_issue_references_accept_crlf_frontmatter() {
    let content = "---\r\nmodule: auth\r\nimplements:\r\n  - 41\r\n  - 42 # inline item comment\r\ntracks: [43, 44]\r\n---\r\n\r\n# Auth\r\n";

    assert_eq!(
        parse_checked_issue_references(content).unwrap(),
        (vec![41, 42], vec![43, 44])
    );
}

#[test]
fn checked_issue_references_keep_crlf_validation_strict() {
    for (content, expected) in [
        (
            "---\r\nmodule: auth\r\nimplements: [41]\r\nimplements: [42]\r\n---\r\n\r\n# Auth\r\n",
            DUPLICATE_IMPLEMENTS_ERROR,
        ),
        (
            "---\r\nmodule: auth\r\nextensions:\r\n  nested: [unterminated\r\n---\r\n\r\n# Auth\r\n",
            INVALID_YAML_FRONTMATTER_ERROR,
        ),
        (
            "---\r\nmodule: auth\r\ntracks: 42\r\n---\r\n\r\n# Auth\r\n",
            INVALID_TRACKS_SHAPE_ERROR,
        ),
    ] {
        assert_eq!(
            parse_checked_issue_references(content).unwrap_err(),
            expected
        );
    }
}

#[test]
fn checked_issue_references_accept_yaml_trailing_commas() {
    let content = "---\nmodule: auth\nimplements: [42,]\ntracks: []\n---\n\n# Auth\n";

    assert_eq!(
        parse_checked_issue_references(content).unwrap(),
        (vec![42], Vec::new())
    );
}

#[test]
fn checked_issue_references_reject_duplicate_keys() {
    for (content, expected) in [
        (
            "---\nmodule: auth\nimplements: [41]\nimplements: [42]\n---\n\n# Auth\n",
            DUPLICATE_IMPLEMENTS_ERROR,
        ),
        (
            "---\nmodule: auth\nextensions:\n  mode: one\n  mode: two\n---\n\n# Auth\n",
            DUPLICATE_YAML_KEY_ERROR,
        ),
    ] {
        assert_eq!(
            parse_checked_issue_references(content).unwrap_err(),
            expected
        );
    }
}

#[test]
fn checked_issue_references_reject_invalid_known_values() {
    let cases = [
        ("implements:", INVALID_IMPLEMENTS_SHAPE_ERROR),
        ("implements: null", INVALID_IMPLEMENTS_SHAPE_ERROR),
        ("implements: 42", INVALID_IMPLEMENTS_SHAPE_ERROR),
        ("implements: [0]", INVALID_IMPLEMENTS_NUMBER_ERROR),
        ("implements: [-1]", INVALID_IMPLEMENTS_NUMBER_ERROR),
        (
            "implements: [18446744073709551616]",
            INVALID_IMPLEMENTS_NUMBER_ERROR,
        ),
        ("implements: [41, nope]", INVALID_IMPLEMENTS_NUMBER_ERROR),
        ("tracks:", INVALID_TRACKS_SHAPE_ERROR),
        ("tracks: null", INVALID_TRACKS_SHAPE_ERROR),
        ("tracks: [41, 0]", INVALID_TRACKS_NUMBER_ERROR),
    ];

    for (field, expected) in cases {
        let content = format!("---\nmodule: auth\n{field}\n---\n\n# Auth\n");
        let result = parse_checked_issue_references(&content);
        assert!(
            result.is_err(),
            "expected rejection for {field}: {result:?}"
        );
        assert_eq!(
            result.unwrap_err(),
            expected,
            "unexpected result for {field}"
        );
    }
}

#[test]
fn checked_issue_references_ignore_nested_extensions_and_block_scalars() {
    let content = "\
---
module: auth
extensions:
  implements: [900]
  nested:
    tracks: invalid
extension_sequence:
  - implements: [901]
    tracks: [902]
notes: |
  implements: invalid
  tracks:
    - 903
folded: >
  tracks: [904]
implements: [41]
tracks: [42]
---

# Auth
";

    assert_eq!(
        parse_checked_issue_references(content).unwrap(),
        (vec![41], vec![42])
    );
}

#[test]
fn checked_issue_references_reject_malformed_unknown_extensions() {
    let content = "---\nmodule: auth\nextensions:\n  nested: [unterminated\n---\n\n# Auth\n";

    assert_eq!(
        parse_checked_issue_references(content).unwrap_err(),
        INVALID_YAML_FRONTMATTER_ERROR
    );
}

#[test]
fn checked_issue_references_reject_reviewer_reproducer_without_leaking_content() {
    let content = "\
---
module: auth
implements:
private_extension:
  secret: [reviewer-reproducer
---

# Auth
";

    let error = parse_checked_issue_references(content).unwrap_err();

    assert!(matches!(
        error.as_str(),
        INVALID_IMPLEMENTS_SHAPE_ERROR | INVALID_YAML_FRONTMATTER_ERROR
    ));
    assert!(!error.contains("reviewer-reproducer"));
    assert!(!error.contains("private_extension"));
}
