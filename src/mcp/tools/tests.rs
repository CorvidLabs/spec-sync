use super::super::snapshot::{ProjectSnapshot, with_isolated_snapshot_git};
use super::super::test_support::{
    assert_no_mcp_transaction_debris, open_test_directory, setup_project, setup_project_with_spec,
};
use super::*;
use crate::config::load_config;
use tempfile::TempDir;

fn git_in(root: &Path, args: &[&str]) {
    let output = std::process::Command::new("git")
        .args(args)
        .current_dir(root)
        .output()
        .expect("git command should run");
    assert!(
        output.status.success(),
        "git {args:?} failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );
}

fn init_git_repo(root: &Path) {
    git_in(root, &["init"]);
    git_in(root, &["config", "user.email", "test@test.com"]);
    git_in(root, &["config", "user.name", "Test"]);
    git_in(root, &["config", "commit.gpgsign", "false"]);
    git_in(root, &["add", "-A"]);
    git_in(root, &["commit", "-m", "init"]);
}

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

// --- load_and_discover ---

#[test]
fn test_load_and_discover_empty_allowed() {
    let tmp = setup_project();
    let result = load_and_discover(tmp.path(), true);
    assert!(result.is_ok());
    let (config, specs) = result.unwrap();
    assert_eq!(config.specs_dir, "specs");
    assert!(specs.is_empty());
}

#[test]
fn test_load_and_discover_empty_not_allowed() {
    let tmp = setup_project();
    let result = load_and_discover(tmp.path(), false);
    assert!(result.is_err());
    assert!(result.unwrap_err().contains("No spec files found"));
}

#[test]
fn test_load_and_discover_filters_private_specs() {
    let tmp = setup_project();
    let spec_dir = tmp.path().join("specs").join("_private");
    std::fs::create_dir_all(&spec_dir).unwrap();
    std::fs::write(
        spec_dir.join("_private.spec.md"),
        "---\nmodule: private\n---",
    )
    .unwrap();

    let result = load_and_discover(tmp.path(), true);
    assert!(result.is_ok());
    let (_config, specs) = result.unwrap();
    // Private specs (starting with _) should be filtered out
    assert!(specs.is_empty());
}

#[test]
fn test_load_and_discover_finds_real_specs() {
    let spec_content = "---\nmodule: auth\nversion: 1.0.0\nstatus: draft\nfiles:\n  - src/auth.rs\n---\n\n# Purpose\nAuth module\n\n# Public API\nNone\n";
    let tmp = setup_project_with_spec("auth", spec_content);

    let result = load_and_discover(tmp.path(), false);
    assert!(result.is_ok());
    let (_config, specs) = result.unwrap();
    assert_eq!(specs.len(), 1);
}

// --- tool_init ---

#[test]
fn test_tool_init_creates_config() {
    let tmp = TempDir::new().unwrap();
    std::fs::create_dir_all(tmp.path().join("src")).unwrap();

    let directory = open_test_directory(tmp.path());
    let result = tool_init(tmp.path(), &directory);
    assert!(
        result.is_ok(),
        "unexpected initialization error: {result:?}"
    );
    let val = result.unwrap();
    assert_eq!(val["created"], true);
    assert!(tmp.path().join("specsync.json").exists());
    assert_no_mcp_transaction_debris(tmp.path());
}

#[test]
fn test_tool_init_already_exists() {
    let tmp = setup_project();
    let directory = open_test_directory(tmp.path());
    let result = tool_init(tmp.path(), &directory);
    assert!(result.is_ok());
    let val = result.unwrap();
    assert_eq!(val["created"], false);
    assert!(val["message"].as_str().unwrap().contains("already exists"));
}

#[test]
fn test_generation_outcome_rejects_missing_required_output() {
    let tmp = TempDir::new().unwrap();
    let expected = vec![(
        "missing".to_string(),
        tmp.path().join("specs/missing/missing.spec.md"),
    )];
    let outcome = crate::generator::GenerationOutcome {
        generated: 1,
        generated_paths: vec!["specs/missing/missing.spec.md".to_string()],
        skipped_no_files: Vec::new(),
    };

    let error = validate_generation_outcome(tmp.path(), &expected, &outcome).unwrap_err();
    assert!(error.contains("did not create the required spec"));
}

#[test]
fn test_generation_outcome_rejects_incomplete_count() {
    let tmp = TempDir::new().unwrap();
    let expected = vec![(
        "missing".to_string(),
        tmp.path().join("specs/missing/missing.spec.md"),
    )];
    let outcome = crate::generator::GenerationOutcome::default();

    let error = validate_generation_outcome(tmp.path(), &expected, &outcome).unwrap_err();
    assert!(error.contains("created 0 of 1 required specs"));
}

// --- tool_coverage ---

#[test]
fn test_tool_coverage_empty_project() {
    let tmp = setup_project();
    let result = tool_coverage(tmp.path());
    assert!(result.is_ok());
    let val = result.unwrap();
    // Same correction as `test_resource_coverage_empty`: zero files
    // measured is `null`, not full coverage (#582).
    assert!(val["file_coverage"].is_null(), "{val}");
    assert!(val["loc_coverage"].is_null(), "{val}");
    assert_eq!(val["files_total"], 0);
}

#[test]
fn test_tool_coverage_with_unspecced_files() {
    let tmp = setup_project();
    // Create a source file without a spec
    std::fs::write(tmp.path().join("src").join("main.rs"), "fn main() {}").unwrap();

    let result = tool_coverage(tmp.path());
    assert!(result.is_ok());
    let val = result.unwrap();
    assert!(val["files_total"].as_u64().unwrap() > 0);
}

#[test]
fn malformed_gradle_makes_every_mcp_coverage_consumer_inconclusive() {
    let tmp = setup_project();
    fs::write(
        tmp.path().join("settings.gradle.kts"),
        "include(\"member\"\n",
    )
    .unwrap();
    fs::write(tmp.path().join("build.gradle.kts"), "plugins {}\n").unwrap();
    let directory = open_test_directory(tmp.path());

    let errors = [
        resource_coverage(tmp.path()).unwrap_err(),
        tool_check(tmp.path(), &json!({})).unwrap_err(),
        tool_coverage(tmp.path()).unwrap_err(),
        tool_generate(tmp.path(), &directory, &json!({})).unwrap_err(),
    ];

    for error in errors {
        assert!(error.contains("Gradle"), "unexpected error: {error}");
        assert!(
            error.contains("inconclusive") || error.contains("Cannot parse"),
            "malformed Gradle must not become a compatibility coverage report: {error}"
        );
    }
}

// --- tool_list_specs ---

#[test]
fn test_tool_list_specs_empty() {
    let tmp = setup_project();
    let result = tool_list_specs(tmp.path());
    assert!(result.is_ok());
    let val = result.unwrap();
    assert_eq!(val["count"], 0);
    assert!(val["specs"].as_array().unwrap().is_empty());
}

#[test]
fn test_tool_list_specs_with_frontmatter() {
    let spec_content = "---\nmodule: auth\nversion: 2.0.0\nstatus: stable\nfiles:\n  - src/auth.rs\n---\n\n# Purpose\nAuth\n";
    let tmp = setup_project_with_spec("auth", spec_content);

    let result = tool_list_specs(tmp.path());
    assert!(result.is_ok());
    let val = result.unwrap();
    assert_eq!(val["count"], 1);
    let spec = &val["specs"][0];
    assert_eq!(spec["module"], "auth");
    assert_eq!(spec["version"], "2.0.0");
    assert_eq!(spec["status"], "stable");
}

#[test]
fn test_tool_list_specs_malformed_frontmatter() {
    let tmp = setup_project();
    let spec_dir = tmp.path().join("specs").join("bad");
    std::fs::create_dir_all(&spec_dir).unwrap();
    std::fs::write(spec_dir.join("bad.spec.md"), "no frontmatter here").unwrap();

    let result = tool_list_specs(tmp.path());
    assert!(result.is_ok());
    let val = result.unwrap();
    assert_eq!(val["count"], 1);
    let spec = &val["specs"][0];
    assert!(spec["module"].is_null());
}

// --- tool_check ---

#[test]
fn test_tool_check_strict_mode() {
    let spec_content = "---\nmodule: auth\nversion: 1.0.0\nstatus: draft\nfiles:\n  - src/auth.rs\n---\n\n# Purpose\nAuth module\n\n# Public API\nNone\n";
    let tmp = setup_project_with_spec("auth", spec_content);
    std::fs::write(tmp.path().join("src").join("auth.rs"), "pub fn login() {}").unwrap();

    let result_normal = tool_check(tmp.path(), &json!({ "strict": false }));
    assert!(result_normal.is_ok());

    let result_strict = tool_check(tmp.path(), &json!({ "strict": true }));
    assert!(result_strict.is_ok());
}

#[test]
fn test_tool_check_no_specs_error() {
    let tmp = setup_project();
    let result = tool_check(tmp.path(), &json!({}));
    assert!(result.is_err());
}

// --- tool_score ---

#[test]
fn test_tool_score_no_specs_error() {
    let tmp = setup_project();
    let result = tool_score(tmp.path(), true);
    assert!(result.is_err());
}

#[test]
fn test_tool_score_with_spec() {
    let spec_content = "---\nmodule: auth\nversion: 1.0.0\nstatus: draft\nfiles:\n  - src/auth.rs\n---\n\n# Purpose\nAuth module\n\n# Public API\nNone\n\n# Invariants\nNone\n\n# Behavioral Examples\nNone\n\n# Error Cases\nNone\n\n# Dependencies\nNone\n\n# Change Log\nNone\n";
    let tmp = setup_project_with_spec("auth", spec_content);
    std::fs::write(tmp.path().join("src").join("auth.rs"), "pub fn login() {}").unwrap();

    let result = tool_score(tmp.path(), true);
    assert!(result.is_ok());
    let val = result.unwrap();
    assert!(val["average_score"].as_f64().unwrap() >= 0.0);
    assert!(val["grade"].is_string());
    assert_eq!(val["total_specs"], 1);
    assert!(val["distribution"].is_object());
}

// --- tool_generate ---

#[test]
fn test_tool_generate_no_uncovered() {
    let spec_content = "---\nmodule: auth\nversion: 1.0.0\nstatus: draft\nfiles:\n  - src/auth.rs\n---\n\n# Purpose\nAuth\n";
    let tmp = setup_project_with_spec("auth", spec_content);
    std::fs::write(tmp.path().join("src").join("auth.rs"), "pub fn login() {}").unwrap();

    let snapshot = ProjectSnapshot::create(tmp.path()).unwrap();
    let directory = open_test_directory(tmp.path());
    let result = tool_generate(snapshot.root(), &directory, &json!({}));
    assert!(result.is_ok());
}

#[test]
fn test_tool_generate_creates_spec() {
    let tmp = setup_project();
    std::fs::write(tmp.path().join("src").join("auth.rs"), "pub fn login() {}").unwrap();

    let snapshot = ProjectSnapshot::create(tmp.path()).unwrap();
    let directory = open_test_directory(tmp.path());
    let result = tool_generate(snapshot.root(), &directory, &json!({}));
    assert!(result.is_ok(), "unexpected generation error: {result:?}");
    let val = result.unwrap();
    assert_eq!(val["count"].as_u64(), Some(1));
    let generated = val["generated"].as_array().unwrap();
    assert_eq!(generated.len(), 1);
    let generated_path = std::path::Path::new(generated[0].as_str().unwrap());
    assert!(!generated_path.is_absolute());
    assert!(tmp.path().join(generated_path).is_file());
    assert!(
        generated_path.ends_with(
            std::path::Path::new("specs")
                .join("auth")
                .join("auth.spec.md")
        )
    );
    assert_no_mcp_transaction_debris(tmp.path());
}

#[test]
fn generation_rejects_an_unbounded_output_count_before_reading_files() {
    let tmp = TempDir::new().unwrap();
    let expected = (0..=MAX_GENERATED_SPECS)
        .map(|index| {
            (
                format!("module-{index}"),
                tmp.path()
                    .join(format!("specs/module-{index}/module-{index}.spec.md")),
            )
        })
        .collect::<Vec<_>>();
    let outcome = crate::generator::GenerationOutcome {
        generated: expected.len(),
        generated_paths: Vec::new(),
        skipped_no_files: Vec::new(),
    };

    let error = validate_generation_outcome(tmp.path(), &expected, &outcome).unwrap_err();
    assert!(error.contains("1000-spec output limit"));
}

#[test]
fn generation_rejects_cumulative_output_bytes_before_publication() {
    let tmp = TempDir::new().unwrap();
    let first = tmp.path().join("specs/first/first.spec.md");
    let second = tmp.path().join("specs/second/second.spec.md");
    fs::create_dir_all(first.parent().unwrap()).unwrap();
    fs::create_dir_all(second.parent().unwrap()).unwrap();
    fs::write(&first, "12345").unwrap();
    fs::write(&second, "67890").unwrap();
    let expected = vec![("first".to_string(), first), ("second".to_string(), second)];
    let outcome = crate::generator::GenerationOutcome {
        generated: 2,
        generated_paths: vec![
            "specs/first/first.spec.md".to_string(),
            "specs/second/second.spec.md".to_string(),
        ],
        skipped_no_files: Vec::new(),
    };

    assert_eq!(MAX_GENERATED_OUTPUT_BYTES, 64 * 1024 * 1024);
    let error = validate_generation_outcome_with_limits(
        tmp.path(),
        &expected,
        &outcome,
        MAX_GENERATED_SPECS,
        8,
    )
    .unwrap_err();
    assert!(error.contains("cumulative output limit"));
}

#[test]
fn test_tool_generate_rejects_retired_ai_arguments_without_echoing_values() {
    let tmp = setup_project();
    let secret = "sk-do-not-echo";
    let directory = open_test_directory(tmp.path());
    let error = tool_generate(tmp.path(), &directory, &json!({ "apiKey": secret })).unwrap_err();
    assert!(error.contains("removed in spec-sync 5.0"));
    assert!(!error.contains(secret));
}

#[test]
fn test_tool_generate_rejects_unknown_arguments() {
    let tmp = setup_project();
    let directory = open_test_directory(tmp.path());
    let error = tool_generate(tmp.path(), &directory, &json!({ "unexpected": true })).unwrap_err();
    assert!(error.contains("Unknown MCP generate argument `unexpected`"));
}

#[test]
fn test_mcp_score_always_marks_git_freshness_unavailable_and_fails_closed() {
    let spec_content = "---\nmodule: auth\nversion: 1.0.0\nstatus: stable\nfiles:\n  - src/auth.rs\n---\n\n# Purpose\nAuthentication behavior.\n\n# Requirements\n- Authenticate users.\n\n# Public API\n| Name | Kind | Description |\n| --- | --- | --- |\n| `login` | Function | Authenticates a user. |\n\n# Invariants\n- Credentials are validated.\n\n# Behavioral Examples\nCalling `login` validates credentials.\n\n# Error Cases\nInvalid credentials return an error.\n\n# Dependencies\nNone.\n\n# Change Log\n- 1.0.0: Initial specification.\n";
    let tmp = setup_project_with_spec("auth", spec_content);
    fs::write(tmp.path().join("src/auth.rs"), "pub fn login() {}\n").unwrap();
    let snapshot = ProjectSnapshot::create(tmp.path()).unwrap();
    assert!(!snapshot.git_freshness_available());
    let config = load_config(snapshot.root());
    let spec_path = snapshot.root().join("specs/auth/auth.spec.md");
    let baseline = scoring::score_spec(&spec_path, snapshot.root(), &config);
    let confined = score_spec_for_mcp(&spec_path, snapshot.root(), &config, false);

    // The snapshot is a temp copy with no `.git`, so `score_spec` itself
    // now withholds the git-freshness budget (#572) — it did not before,
    // and this wrapper was the only thing standing between an agent and 5
    // points for a question git could not be asked. The invariant that
    // matters is unchanged and now holds twice over: NOTHING in the
    // confined path awards those points. What must not happen is charging
    // for the same absence twice, so `confined` equals `baseline` here.
    assert_eq!(baseline.git_freshness, scoring::GitFreshness::Withheld);
    assert_eq!(
        confined.total, baseline.total,
        "the confined score must not be penalized a second time for one absence"
    );
    assert_eq!(confined.freshness_score, baseline.freshness_score);
    let git_criterion = confined
        .explain
        .iter()
        .find(|detail| detail.dimension == "Freshness")
        .and_then(|detail| {
            detail
                .criteria
                .iter()
                .find(|criterion| criterion.name == "git_freshness")
        })
        .expect("freshness/git_freshness criterion");
    assert_eq!(
        git_criterion.points, 0,
        "the confined snapshot must award no git-freshness points"
    );
    assert!(!git_criterion.passed);
    assert!(
        confined
            .suggestions
            .iter()
            .any(|suggestion| suggestion.contains("Git history is intentionally unavailable"))
    );
    assert!(
        !confined
            .suggestions
            .iter()
            .any(|suggestion| suggestion.contains("drift is unverifiable")),
        "the confined wording replaces the generic one rather than stacking with it: {:?}",
        confined.suggestions
    );
    let output = tool_score(snapshot.root(), snapshot.git_freshness_available()).unwrap();
    assert_eq!(output["git_freshness_available"], false);
    assert_eq!(output["specs"][0]["git_freshness_available"], false);
}

#[test]
fn mcp_snapshot_inside_host_worktree_does_not_discover_host_git_or_double_penalize() {
    let spec_content = "---\nmodule: auth\nversion: 1.0.0\nstatus: stable\nfiles:\n  - src/auth.rs\n---\n\n# Purpose\nAuthentication behavior.\n\n# Requirements\n- Authenticate users.\n\n# Public API\n| Name | Kind | Description |\n| --- | --- | --- |\n| `login` | Function | Authenticates a user. |\n\n# Invariants\n- Credentials are validated.\n\n# Behavioral Examples\nCalling `login` validates credentials.\n\n# Error Cases\nInvalid credentials return an error.\n\n# Dependencies\nNone.\n\n# Change Log\n- 1.0.0: Initial specification.\n";
    let host = TempDir::new().unwrap();
    let project = host.path().join("project");
    fs::create_dir_all(project.join("src")).unwrap();
    fs::create_dir_all(project.join("specs/auth")).unwrap();
    let config = json!({
        "specsDir": "specs",
        "sourceDirs": ["src"],
        "requiredSections": ["Purpose", "Public API"]
    });
    fs::write(
        project.join("specsync.json"),
        serde_json::to_string_pretty(&config).unwrap(),
    )
    .unwrap();
    fs::write(project.join("src/auth.rs"), "pub fn login() {}\n").unwrap();
    fs::write(project.join("specs/auth/auth.spec.md"), spec_content).unwrap();
    init_git_repo(host.path());

    let temp_parent = host.path().join("tmp");
    fs::create_dir_all(&temp_parent).unwrap();
    let snapshot = ProjectSnapshot::create_in(&project, &temp_parent).unwrap();
    assert!(
        crate::git_utils::is_git_repo(snapshot.root()),
        "without isolation, a snapshot whose tempfile sits in the host worktree discovers that work tree"
    );
    assert!(!snapshot.git_freshness_available());

    let config = load_config(snapshot.root());
    let spec_path = snapshot.root().join("specs/auth/auth.spec.md");
    let (baseline, confined, output) = with_isolated_snapshot_git(&snapshot, || {
        let baseline = scoring::score_spec(&spec_path, snapshot.root(), &config);
        let confined = score_spec_for_mcp(&spec_path, snapshot.root(), &config, false);
        let output = tool_score(snapshot.root(), snapshot.git_freshness_available()).unwrap();
        (baseline, confined, output)
    });

    assert_eq!(baseline.git_freshness, scoring::GitFreshness::Withheld);
    assert_eq!(
        confined.total, baseline.total,
        "host walk-up must not award git freshness that the confined path then deducts again"
    );
    assert_eq!(confined.freshness_score, baseline.freshness_score);
    assert_eq!(output["git_freshness_available"], false);
    assert_eq!(output["specs"][0]["git_freshness_available"], false);
    assert!(
        confined
            .suggestions
            .iter()
            .any(|suggestion| suggestion.contains("Git history is intentionally unavailable"))
    );
}
