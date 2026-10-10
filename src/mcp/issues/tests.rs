use super::*;
#[cfg(target_os = "linux")]
use crate::mcp::ProjectSnapshot;
use crate::mcp::tests::{setup_project, setup_project_with_spec};
use std::fs;

#[test]
fn issue_verification_errors_fail_closed_instead_of_returning_empty_success() {
    let verification = crate::github::IssueVerification {
        spec_path: "specs/mcp/mcp.spec.md".to_string(),
        valid: Vec::new(),
        closed: Vec::new(),
        not_found: Vec::new(),
        errors: vec!["#414: authentication failed".to_string()],
    };

    let error =
        ensure_issue_verification_complete("specs/mcp/mcp.spec.md", &verification).unwrap_err();
    assert!(error.contains("inconclusive"));
    assert!(error.contains("#414"));
}

#[test]
fn issue_tool_enforces_one_deduplicated_invocation_cap_across_specs() {
    let tmp = setup_project();
    fs::write(
        tmp.path().join("specsync.json"),
        r#"{"specsDir":"specs","sourceDirs":["src"],"github":{"repo":"CorvidLabs/spec-sync"}}"#,
    )
    .unwrap();
    let first: Vec<String> = (1..=60).map(|number| number.to_string()).collect();
    let second: Vec<String> = (41..=101).map(|number| number.to_string()).collect();
    for (module, issues) in [("one", first), ("two", second)] {
        let directory = tmp.path().join("specs").join(module);
        fs::create_dir_all(&directory).unwrap();
        fs::write(
                directory.join(format!("{module}.spec.md")),
                format!(
                    "---\nmodule: {module}\nversion: 1\nstatus: draft\nfiles: []\ntracks: [{}]\n---\n\n# Purpose\nTest\n",
                    issues.join(", ")
                ),
            )
            .unwrap();
    }

    let error = tool_issues(tmp.path())
        .expect_err("the global issue cap must fail before provider selection");

    assert!(error.contains("100-issue invocation limit"));
}

#[test]
fn issue_tool_fails_inconclusive_for_malformed_frontmatter() {
    let tmp = setup_project_with_spec("malformed", "# Missing frontmatter\n");

    let error = tool_issues(tmp.path())
        .expect_err("malformed frontmatter must not produce a zero-reference success");

    assert_eq!(
        error,
        "MCP specsync_issues issue discovery is inconclusive for specs/malformed/malformed.spec.md: missing or malformed YAML frontmatter"
    );
}

#[test]
fn issue_tool_fails_inconclusive_for_unreadable_spec_text() {
    let tmp = setup_project();
    let spec_dir = tmp.path().join("specs").join("unreadable");
    fs::create_dir_all(&spec_dir).unwrap();
    let spec_path = spec_dir.join("unreadable.spec.md");
    fs::write(&spec_path, [0xff]).unwrap();

    let error = tool_issues(tmp.path())
        .expect_err("unreadable spec text must not produce a zero-reference success");

    assert_eq!(
        error,
        "MCP specsync_issues issue discovery is inconclusive for specs/unreadable/unreadable.spec.md: spec file is not valid UTF-8"
    );
    assert!(!error.contains(&tmp.path().to_string_lossy().to_string()));
}

#[test]
fn issue_tool_fails_inconclusive_for_malformed_known_issue_fields() {
    let cases = [
        ("implements:", "`implements` must be a list"),
        ("implements: null", "`implements` must be a list"),
        ("implements: nope", "`implements` must be a list"),
        ("tracks: 42", "`tracks` must be a list"),
        ("implements : [nope]", "`implements` contains an invalid"),
        ("implements: [0]", "`implements` contains an invalid"),
        ("implements: [-1]", "`implements` contains an invalid"),
        (
            "implements: [18446744073709551616]",
            "`implements` contains an invalid",
        ),
        (
            "implements: [41, not-a-number, 42]",
            "`implements` contains an invalid unsigned issue number",
        ),
        (
            "tracks:\n  - 41\n  - nope\n  - 42",
            "`tracks` contains an invalid unsigned issue number",
        ),
        (
            "tracks:\n- 41\n- nope",
            "`tracks` contains an invalid unsigned issue number",
        ),
    ];

    for (field, expected_reason) in cases {
        let spec = format!(
            "---\nmodule: malformed\nversion: 1\nstatus: draft\nfiles: []\n{field}\n---\n\n# Purpose\nTest\n"
        );
        let tmp = setup_project_with_spec("malformed", &spec);

        let error = tool_issues(tmp.path())
            .expect_err("malformed known issue fields must not produce a zero-reference success");

        assert!(
            error.contains(expected_reason),
            "unexpected error for {field}: {error}"
        );
        assert!(!error.contains(&tmp.path().to_string_lossy().to_string()));
    }
}

#[test]
fn issue_reference_field_validation_accepts_supported_list_forms() {
    let content = "---\nmodule: valid\nimplements: [41, 42]\ntracks:\n  - 43\n  - 44 # retained comment\n---\n\n# Purpose\nTest\n";

    assert_eq!(
        crate::parser::parse_checked_issue_references(content).unwrap(),
        (vec![41, 42], vec![43, 44])
    );
}

#[test]
fn issue_discovery_accepts_yaml_trailing_comma_and_inline_comment() {
    let content = "---\nmodule: valid\nversion: 1\nstatus: draft\nfiles: []\nimplements: [42,] # valid YAML\n---\n\n# Purpose\nTest\n";
    let tmp = setup_project_with_spec("valid", content);

    let (_, references) = discover_issue_references(tmp.path()).unwrap();

    assert_eq!(
        references,
        vec![(
            "specs/valid/valid.spec.md".to_string(),
            vec![42],
            Vec::new()
        )]
    );
}

#[test]
fn issue_discovery_accepts_crlf_frontmatter_and_retains_references() {
    let content = "---\r\nmodule: crlf\r\nversion: 1\r\nstatus: draft\r\nfiles: []\r\nimplements: [41, 42]\r\ntracks:\r\n  - 43\r\n---\r\n\r\n# Purpose\r\nTest\r\n";
    let tmp = setup_project_with_spec("crlf", content);

    let (_, references) = discover_issue_references(tmp.path()).unwrap();

    assert_eq!(
        references,
        vec![(
            "specs/crlf/crlf.spec.md".to_string(),
            vec![41, 42],
            vec![43],
        )]
    );
}

#[test]
fn issue_reference_field_validation_ignores_nested_extensions_and_block_scalars() {
    let content = "\
---
module: valid
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
implements: [41, 42]
tracks:
  - 43
  - 44
---

# Purpose
Test
";

    assert_eq!(
        crate::parser::parse_checked_issue_references(content).unwrap(),
        (vec![41, 42], vec![43, 44])
    );
}

#[test]
fn issue_reference_field_validation_keeps_top_level_known_fields_strict() {
    let content = "\
---
module: invalid
extensions:
  implements: [900]
implements:
  nested: invalid
---

# Purpose
Test
";

    assert_eq!(
        crate::parser::parse_checked_issue_references(content).unwrap_err(),
        "`implements` must be a list of unsigned issue numbers"
    );
}

#[test]
fn issue_tool_rejects_duplicate_issue_reference_keys() {
    let content = "---\nmodule: invalid\nversion: 1\nstatus: draft\nfiles: []\nimplements: [41]\nimplements: [42]\n---\n\n# Purpose\nTest\n";
    let tmp = setup_project_with_spec("invalid", content);

    let error = tool_issues(tmp.path()).unwrap_err();

    assert_eq!(
        error,
        "MCP specsync_issues issue discovery is inconclusive for specs/invalid/invalid.spec.md: duplicate `implements` issue-reference field"
    );
}

#[test]
fn issue_tool_rejects_reviewer_reproducer_without_leaking_content() {
    let content = "\
---
module: invalid
version: 1
status: draft
files: []
implements:
private_extension:
  secret: [reviewer-reproducer
---

# Purpose
Test
";
    let tmp = setup_project_with_spec("invalid", content);

    let error = tool_issues(tmp.path()).unwrap_err();

    assert!(error.contains("issue discovery is inconclusive"));
    assert!(
        error.contains("invalid YAML frontmatter") || error.contains("`implements` must be a list")
    );
    assert!(!error.contains("reviewer-reproducer"));
    assert!(!error.contains("private_extension"));
}

#[cfg(unix)]
#[test]
fn issue_spec_file_name_rejects_non_utf8_spec_suffix() {
    use std::ffi::OsString;
    use std::os::unix::ffi::OsStringExt;

    let mut file_name = b"hidden-".to_vec();
    file_name.push(0xff);
    file_name.extend_from_slice(b".spec.md");

    assert_eq!(
        is_issue_spec_file_name(&OsString::from_vec(file_name)),
        Err(())
    );
}

#[cfg(target_os = "linux")]
#[test]
fn issue_tool_rejects_non_utf8_spec_filename_after_snapshot_copy() {
    use std::ffi::OsString;
    use std::os::unix::ffi::OsStringExt;

    let tmp = setup_project();
    let spec_dir = tmp.path().join("specs").join("hostile");
    fs::create_dir_all(&spec_dir).unwrap();
    let mut file_name = b"hidden-".to_vec();
    file_name.push(0xff);
    file_name.extend_from_slice(b".spec.md");
    let spec_path = spec_dir.join(OsString::from_vec(file_name));
    fs::write(
        &spec_path,
        "---\nmodule: hostile\nversion: 1\nstatus: draft\nfiles: []\n---\n\n# Purpose\nTest\n",
    )
    .unwrap();

    let snapshot = ProjectSnapshot::create(tmp.path()).unwrap();
    let error = tool_issues(snapshot.root())
        .expect_err("a non-UTF-8 spec filename must make issue discovery inconclusive");

    assert!(error.contains("spec filename is not valid UTF-8"));
    assert!(error.contains("specs/hostile/hidden-"));
    assert!(!error.contains(&tmp.path().to_string_lossy().to_string()));
    assert!(!error.contains("# Purpose"));
    assert!(error.chars().count() < 400);
}

#[test]
fn issue_read_diagnostics_are_bounded_relative_and_content_free() {
    let tmp = setup_project();
    let long_name = "x".repeat(MAX_ISSUE_DIAGNOSTIC_PATH_CHARS + 80);
    let path = tmp.path().join("specs").join(long_name);

    let relative = issue_relative_spec_path(tmp.path(), &path);
    let reason = issue_spec_read_error_reason(&format!(
        "Cannot read MCP spec file {}: secret operating system detail",
        path.display()
    ));

    assert!(relative.chars().count() <= MAX_ISSUE_DIAGNOSTIC_PATH_CHARS);
    assert!(!relative.contains(&tmp.path().to_string_lossy().to_string()));
    assert_eq!(reason, "spec file could not be read");
    assert!(!reason.contains("secret"));
}

#[test]
fn issue_relative_spec_path_escapes_every_unsafe_display_character() {
    let root = Path::new("project");
    let dangerous_code_points = (0x0000u32..=0x001f)
        .chain(0x007f..=0x009f)
        .chain([0x061c, 0x200e, 0x200f, 0x2028, 0x2029])
        .chain(0x202a..=0x202e)
        .chain(0x2066..=0x2069);

    for code_point in dangerous_code_points {
        let character = char::from_u32(code_point).expect("test code point must be valid Unicode");
        let path = root
            .join("specs")
            .join(format!("before{character}after.spec.md"));
        let escaped = format!("\\u{{{code_point:04X}}}");

        assert_eq!(
            issue_relative_spec_path(root, &path),
            format!("specs/before{escaped}after.spec.md"),
            "unsafe diagnostic character U+{code_point:04X} was not escaped"
        );
    }

    let path = root.join(r"specs\windows\path.spec.md");
    #[cfg(windows)]
    assert_eq!(
        issue_relative_spec_path(root, &path),
        "specs/windows/path.spec.md"
    );
    #[cfg(not(windows))]
    assert_eq!(
        issue_relative_spec_path(root, &path),
        r"specs\windows\path.spec.md"
    );
}

#[cfg(unix)]
#[test]
fn issue_tool_escapes_adversarial_filename_formatting_in_diagnostic() {
    let tmp = setup_project();
    let spec_dir = tmp.path().join("specs").join("hostile");
    fs::create_dir_all(&spec_dir).unwrap();
    let dangerous_characters = [
        '\n', '\u{001b}', '\u{009b}', '\u{061c}', '\u{200e}', '\u{200f}', '\u{2028}', '\u{2029}',
        '\u{202a}', '\u{202b}', '\u{202c}', '\u{202d}', '\u{202e}', '\u{2066}', '\u{2067}',
        '\u{2068}', '\u{2069}',
    ];
    let hostile: String = dangerous_characters.iter().copied().collect();
    let filename = format!(r"segment\before{hostile}after.spec.md");
    fs::write(spec_dir.join(filename), "# Missing frontmatter\n").unwrap();

    let error = tool_issues(tmp.path())
        .expect_err("unsafe filename formatting must not produce a successful issue result");

    assert!(error.contains(r"specs/hostile/segment\before"));
    for character in dangerous_characters {
        assert!(
            error.contains(&format!("\\u{{{:04X}}}", character as u32)),
            "diagnostic omitted escape for U+{:04X}",
            character as u32
        );
        assert!(
            !error.contains(character),
            "diagnostic retained unsafe character U+{:04X}",
            character as u32
        );
    }
    assert!(error.ends_with("after.spec.md: missing or malformed YAML frontmatter"));
}
