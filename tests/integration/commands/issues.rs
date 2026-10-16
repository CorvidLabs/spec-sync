use crate::helpers::{setup_minimal_project, specsync, valid_spec};
use predicates::prelude::*;
use std::fs;
use tempfile::TempDir;

#[cfg(windows)]
use super::create_windows_junction;

#[test]
fn issues_without_references_does_not_require_repository_configuration() {
    let tmp = TempDir::new().unwrap();
    let root = setup_minimal_project(&tmp);

    specsync()
        .arg("issues")
        .arg("--root")
        .arg(&root)
        .assert()
        .success()
        .stdout(predicate::str::contains(
            "No issue references found in spec frontmatter.",
        ))
        .stdout(predicate::str::contains("Verifying issue references").not())
        .stderr(predicate::str::contains("Cannot determine GitHub repo").not());
}

#[test]
fn issues_without_references_preserves_configured_repository_outputs() {
    let tmp = TempDir::new().unwrap();
    let root = setup_minimal_project(&tmp);
    let config_path = root.join("specsync.json");
    let config = fs::read_to_string(&config_path).unwrap();
    fs::write(
        config_path,
        config.replace(
            "\n}",
            ",\n  \"github\": { \"repo\": \"CorvidLabs/spec-sync\" }\n}",
        ),
    )
    .unwrap();

    specsync()
        .arg("issues")
        .arg("--root")
        .arg(&root)
        .assert()
        .success()
        .stdout(predicate::str::contains(
            "No issue references found in spec frontmatter.",
        ))
        .stdout(predicate::str::contains("Verifying issue references").not())
        .stderr(predicate::str::contains("Cannot determine GitHub repo").not());

    specsync()
        .arg("issues")
        .arg("--root")
        .arg(&root)
        .args(["--format", "json"])
        .assert()
        .success()
        .stdout(predicate::str::contains(
            "\"repo\": \"CorvidLabs/spec-sync\"",
        ))
        .stdout(predicate::str::contains("\"valid\": 0"))
        .stdout(predicate::str::contains("\"errors\": 0"))
        .stdout(predicate::str::contains("\"specs\": []"))
        .stderr(predicate::str::contains("Cannot determine GitHub repo").not());
}

#[test]
fn issues_fails_closed_when_malformed_config_hides_custom_specs() {
    let tmp = TempDir::new().unwrap();
    let root = setup_minimal_project(&tmp);
    fs::remove_dir_all(root.join("specs")).unwrap();
    let custom_specs = root.join("custom-specs/auth");
    fs::create_dir_all(&custom_specs).unwrap();
    fs::write(
        custom_specs.join("auth.spec.md"),
        valid_spec("auth", &["src/auth/service.ts"])
            .replace("depends_on: []", "depends_on: []\nimplements: [42]"),
    )
    .unwrap();
    fs::write(
        root.join("specsync.json"),
        r#"{"specsDir":"custom-specs","sourceDirs":["src"]"#,
    )
    .unwrap();

    specsync()
        .arg("issues")
        .arg("--root")
        .arg(&root)
        .assert()
        .failure()
        .stdout(predicate::str::contains("<project-config>"))
        .stdout(predicate::str::contains(
            "Unable to read or parse project configuration.",
        ))
        .stdout(predicate::str::contains("No spec files found.").not())
        .stdout(predicate::str::contains("No issue references found in spec frontmatter.").not());

    let output = specsync()
        .arg("issues")
        .arg("--root")
        .arg(&root)
        .args(["--format", "json"])
        .assert()
        .failure()
        .get_output()
        .stdout
        .clone();
    let json: serde_json::Value =
        serde_json::from_slice(&output).expect("config failure must preserve structured JSON");
    assert_eq!(json["inspection_findings"], 1);
    assert_eq!(json["findings"][0]["kind"], "configuration_error");
    assert_eq!(json["findings"][0]["spec"], "<project-config>");
    assert_eq!(
        json["findings"][0]["message"],
        "Unable to read or parse project configuration."
    );
}

#[test]
fn issues_fails_closed_when_config_is_not_readable_text() {
    let tmp = TempDir::new().unwrap();
    let root = setup_minimal_project(&tmp);
    fs::remove_dir_all(root.join("specs")).unwrap();
    let custom_specs = root.join("custom-specs/auth");
    fs::create_dir_all(&custom_specs).unwrap();
    fs::write(
        custom_specs.join("auth.spec.md"),
        valid_spec("auth", &["src/auth/service.ts"]),
    )
    .unwrap();
    fs::write(
        root.join("specsync.json"),
        b"{\"specsDir\":\"custom-specs\",\"sourceDirs\":[\"src\"]}\xff",
    )
    .unwrap();

    let output = specsync()
        .arg("issues")
        .arg("--root")
        .arg(&root)
        .args(["--format", "json"])
        .assert()
        .failure()
        .get_output()
        .stdout
        .clone();
    let json: serde_json::Value =
        serde_json::from_slice(&output).expect("config failure must preserve structured JSON");
    assert_eq!(json["inspection_findings"], 1);
    assert_eq!(json["findings"][0]["kind"], "configuration_error");
    assert_eq!(json["findings"][0]["spec"], "<project-config>");
    assert_eq!(
        json["findings"][0]["message"],
        "Unable to read or parse project configuration."
    );
}

#[cfg(unix)]
#[test]
fn issues_rejects_symlinked_selected_config_without_reading_target() {
    use std::os::unix::fs::symlink;

    let tmp = TempDir::new().unwrap();
    let root = setup_minimal_project(&tmp);
    let config_path = root.join("specsync.json");
    let outside_config = tmp.path().join("outside-config.json");
    let outside_bytes =
        br#"{"specsDir":"specs","sourceDirs":["src"],"github":{"repo":"CorvidLabs/spec-sync"}}"#;
    fs::write(&outside_config, outside_bytes).unwrap();
    fs::remove_file(&config_path).unwrap();
    symlink(&outside_config, &config_path).unwrap();

    let output = specsync()
        .arg("issues")
        .arg("--root")
        .arg(&root)
        .args(["--format", "json"])
        .assert()
        .failure()
        .get_output()
        .stdout
        .clone();
    let json: serde_json::Value =
        serde_json::from_slice(&output).expect("config rejection must preserve structured JSON");
    assert_eq!(json["inspection_findings"], 1);
    assert_eq!(json["findings"][0]["kind"], "configuration_error");
    assert_eq!(json["findings"][0]["spec"], "<project-config>");
    assert_eq!(fs::read(outside_config).unwrap(), outside_bytes);
}

#[test]
fn issues_bounds_selected_config_snapshots() {
    const CONFIG_LIMIT: usize = 4 * 1024 * 1024;

    let tmp = TempDir::new().unwrap();
    let root = setup_minimal_project(&tmp);
    let padding = "x".repeat(CONFIG_LIMIT);
    fs::write(
        root.join("specsync.json"),
        format!("{{\"specsDir\":\"specs\",\"sourceDirs\":[\"src\"],\"padding\":\"{padding}\"}}"),
    )
    .unwrap();

    let output = specsync()
        .arg("issues")
        .arg("--root")
        .arg(&root)
        .args(["--format", "json"])
        .assert()
        .failure()
        .get_output()
        .stdout
        .clone();
    let json: serde_json::Value =
        serde_json::from_slice(&output).expect("bounded config failure must be structured JSON");
    assert_eq!(json["inspection_findings"], 1);
    assert_eq!(json["findings"][0]["kind"], "configuration_error");
    assert_eq!(json["findings"][0]["spec"], "<project-config>");
}

#[test]
fn issues_rejects_wrong_shaped_toml_path_fields_from_retained_snapshot() {
    for malformed_field in [
        "specs_dir = [\"specs\"]",
        "source_dirs = \"src\"",
        "schema_dir = [\"schema\"]",
    ] {
        let tmp = TempDir::new().unwrap();
        let root = setup_minimal_project(&tmp);
        fs::create_dir_all(root.join(".specsync")).unwrap();
        fs::write(
            root.join(".specsync/config.toml"),
            format!("{malformed_field}\n"),
        )
        .unwrap();

        let output = specsync()
            .arg("issues")
            .arg("--root")
            .arg(&root)
            .args(["--format", "json"])
            .assert()
            .failure()
            .get_output()
            .stdout
            .clone();
        let json: serde_json::Value = serde_json::from_slice(&output)
            .expect("wrong-shaped TOML path fields must produce structured JSON");
        assert_eq!(json["inspection_findings"], 1);
        assert_eq!(json["findings"][0]["kind"], "configuration_error");
        assert_eq!(json["findings"][0]["spec"], "<project-config>");
    }
}

#[test]
fn issues_missing_or_empty_specs_use_selected_structured_renderer() {
    for empty_directory in [false, true] {
        for format in ["json", "markdown", "github"] {
            let tmp = TempDir::new().unwrap();
            let root = setup_minimal_project(&tmp);
            fs::remove_dir_all(root.join("specs")).unwrap();
            if empty_directory {
                fs::create_dir_all(root.join("specs")).unwrap();
            }

            let output = specsync()
                .arg("issues")
                .arg("--root")
                .arg(&root)
                .args(["--format", format])
                .assert()
                .success()
                .get_output()
                .stdout
                .clone();

            if format == "json" {
                let json: serde_json::Value = serde_json::from_slice(&output)
                    .expect("missing specs must preserve structured JSON");
                assert_eq!(json["valid"], 0);
                assert_eq!(json["closed"], 0);
                assert_eq!(json["not_found"], 0);
                assert_eq!(json["errors"], 0);
                assert_eq!(json["specs"].as_array().map(Vec::len), Some(0));
            } else {
                let rendered = String::from_utf8(output).unwrap();
                assert!(rendered.contains("## Issue Verification"));
                assert!(rendered.contains("No spec files found."));
            }
        }
    }
}

#[test]
fn issues_repository_resolution_failures_use_selected_structured_renderer() {
    for format in ["json", "markdown", "github"] {
        let tmp = TempDir::new().unwrap();
        let root = setup_minimal_project(&tmp);
        let config_path = root.join("specsync.json");
        let config = fs::read_to_string(&config_path).unwrap();
        fs::write(
            config_path,
            config.replace(
                "\n}",
                ",\n  \"github\": { \"repo\": \"owner/repo/extra\" }\n}",
            ),
        )
        .unwrap();

        let output = specsync()
            .arg("issues")
            .arg("--root")
            .arg(&root)
            .args(["--format", format])
            .assert()
            .failure()
            .get_output()
            .stdout
            .clone();

        if format == "json" {
            let json: serde_json::Value = serde_json::from_slice(&output)
                .expect("repository failure must preserve structured JSON");
            assert!(
                json["error"]
                    .as_str()
                    .is_some_and(|error| error.contains("GitHub repository"))
            );
            assert_eq!(json["specs"].as_array().map(Vec::len), Some(0));
        } else {
            let rendered = String::from_utf8(output).unwrap();
            assert!(rendered.contains("## Issue Verification"));
            assert!(rendered.contains("GitHub repository"));
        }
    }
}

#[test]
fn issues_validates_configured_repository_when_specs_are_missing_or_empty() {
    for empty_directory in [false, true] {
        let tmp = TempDir::new().unwrap();
        let root = setup_minimal_project(&tmp);
        let config_path = root.join("specsync.json");
        let config = fs::read_to_string(&config_path).unwrap();
        fs::write(
            config_path,
            config.replace(
                "\n}",
                ",\n  \"github\": { \"repo\": \"owner/repo/extra\" }\n}",
            ),
        )
        .unwrap();
        fs::remove_dir_all(root.join("specs")).unwrap();
        if empty_directory {
            fs::create_dir_all(root.join("specs")).unwrap();
        }

        specsync()
            .arg("issues")
            .arg("--root")
            .arg(&root)
            .assert()
            .failure()
            .stderr(predicate::str::contains("GitHub repository"))
            .stdout(predicate::str::contains("No spec files found.").not());
    }
}

#[test]
fn issues_reference_batch_fails_closed_without_a_rest_token() {
    let tmp = TempDir::new().unwrap();
    let root = setup_minimal_project(&tmp);
    let config_path = root.join("specsync.json");
    let config = fs::read_to_string(&config_path).unwrap();
    fs::write(
        config_path,
        config.replace(
            "\n}",
            ",\n  \"github\": { \"repo\": \"CorvidLabs/spec-sync\" }\n}",
        ),
    )
    .unwrap();
    let spec_path = root.join("specs/auth/auth.spec.md");
    let spec = fs::read_to_string(&spec_path).unwrap();
    fs::write(
        spec_path,
        spec.replace("depends_on: []", "depends_on: []\nimplements: [42]"),
    )
    .unwrap();

    let output = specsync()
        .arg("issues")
        .arg("--root")
        .arg(&root)
        .args(["--format", "json"])
        .env_remove("GITHUB_TOKEN")
        .assert()
        .failure()
        .get_output()
        .stdout
        .clone();
    let json: serde_json::Value = serde_json::from_slice(&output).unwrap();
    assert_eq!(json["repo"], "CorvidLabs/spec-sync");
    assert_eq!(json["valid"], 0);
    assert_eq!(json["errors"], 1);
    assert_eq!(json["specs"].as_array().map(Vec::len), Some(1));
    assert!(
        json["specs"][0]["errors"][0]
            .as_str()
            .is_some_and(|error| error.contains("GITHUB_TOKEN"))
    );
}

#[test]
fn issues_retains_unreadable_and_malformed_specs_as_safe_findings() {
    let tmp = TempDir::new().unwrap();
    let root = setup_minimal_project(&tmp);
    let malformed_secret = "MALFORMED_SPEC_SECRET_DO_NOT_PRINT";
    fs::write(
        root.join("specs/auth/auth.spec.md"),
        format!("not frontmatter\n{malformed_secret}\n"),
    )
    .unwrap();
    let unreadable_path = root.join("specs/billing/billing.spec.md");
    fs::create_dir_all(unreadable_path.parent().unwrap()).unwrap();
    let unreadable_secret = b"UNREADABLE_SPEC_SECRET_DO_NOT_PRINT";
    let mut unreadable_content = unreadable_secret.to_vec();
    unreadable_content.push(0xff);
    fs::write(&unreadable_path, unreadable_content).unwrap();

    let text = specsync()
        .arg("issues")
        .arg("--root")
        .arg(&root)
        .assert()
        .failure()
        .stdout(predicate::str::contains("auth.spec.md"))
        .stdout(predicate::str::contains("billing.spec.md"))
        .stdout(predicate::str::contains(
            "Malformed or missing spec frontmatter.",
        ))
        .stdout(predicate::str::contains("Unable to read spec file."))
        .stdout(predicate::str::contains("2 spec inspection findings"))
        .stdout(predicate::str::contains("No issue references found in spec frontmatter.").not())
        .get_output()
        .stdout
        .clone();
    let text = String::from_utf8(text).unwrap();
    assert!(!text.contains(malformed_secret));
    assert!(!text.contains(std::str::from_utf8(unreadable_secret).unwrap()));

    let json_output = specsync()
        .arg("issues")
        .arg("--root")
        .arg(&root)
        .args(["--format", "json"])
        .assert()
        .failure()
        .get_output()
        .stdout
        .clone();
    let json: serde_json::Value =
        serde_json::from_slice(&json_output).expect("JSON output must be one parseable document");
    assert_eq!(json["inspection_findings"], 2);
    assert_eq!(json["findings"].as_array().map(Vec::len), Some(2));
    assert_eq!(json["specs"].as_array().map(Vec::len), Some(0));
    assert!(
        json["findings"]
            .as_array()
            .is_some_and(|findings| findings.iter().any(|finding| {
                finding["kind"] == "malformed_frontmatter"
                    && finding["spec"] == "specs/auth/auth.spec.md"
            }))
    );
    assert!(
        json["findings"]
            .as_array()
            .is_some_and(|findings| findings.iter().any(|finding| {
                finding["kind"] == "read_error"
                    && finding["spec"] == "specs/billing/billing.spec.md"
            }))
    );
    let json_text = String::from_utf8(json_output).unwrap();
    assert!(!json_text.contains(malformed_secret));
    assert!(!json_text.contains(std::str::from_utf8(unreadable_secret).unwrap()));

    for format in ["markdown", "github"] {
        let output = specsync()
            .arg("issues")
            .arg("--root")
            .arg(&root)
            .args(["--format", format])
            .assert()
            .failure()
            .stdout(predicate::str::contains("| Inspection findings | 2 |"))
            .stdout(predicate::str::contains("### Spec Inspection Findings"))
            .stdout(predicate::str::contains("auth.spec.md"))
            .stdout(predicate::str::contains("billing.spec.md"))
            .stdout(
                predicate::str::contains("No issue references found in spec frontmatter.").not(),
            )
            .get_output()
            .stdout
            .clone();
        let output = String::from_utf8(output).unwrap();
        assert!(!output.contains(malformed_secret));
        assert!(!output.contains(std::str::from_utf8(unreadable_secret).unwrap()));
    }
}

#[test]
fn issues_rejects_malformed_known_issue_fields_in_every_format() {
    let tmp = TempDir::new().unwrap();
    let root = setup_minimal_project(&tmp);
    let cases = [
        ("auth", "implements: [42, MIXED_INVALID_ISSUE_SECRET]"),
        ("billing", "tracks: WRONG_SCALAR_SHAPE_SECRET"),
        (
            "reporting",
            "implements:\n  nested: WRONG_MAPPING_SHAPE_SECRET",
        ),
    ];
    for (module, issue_field) in cases {
        let spec_dir = root.join("specs").join(module);
        fs::create_dir_all(&spec_dir).unwrap();
        let spec = valid_spec(module, &["src/auth/service.ts"])
            .replace("depends_on: []", &format!("depends_on: []\n{issue_field}"));
        fs::write(spec_dir.join(format!("{module}.spec.md")), spec).unwrap();
    }

    for format in ["text", "table", "csv", "json", "markdown", "github"] {
        let output = specsync()
            .arg("issues")
            .arg("--root")
            .arg(&root)
            .args(["--format", format])
            .assert()
            .failure()
            .get_output()
            .stdout
            .clone();
        let output_text = String::from_utf8(output.clone()).unwrap();
        assert!(
            !output_text.contains("No issue references found in spec frontmatter."),
            "{format} must not report a trustworthy empty-reference result"
        );
        assert!(!output_text.contains("MIXED_INVALID_ISSUE_SECRET"));
        assert!(!output_text.contains("WRONG_SCALAR_SHAPE_SECRET"));
        assert!(!output_text.contains("WRONG_MAPPING_SHAPE_SECRET"));

        if format == "json" {
            let json: serde_json::Value = serde_json::from_slice(&output).unwrap();
            assert_eq!(json["inspection_findings"], 3);
            assert!(json["findings"].as_array().is_some_and(|findings| {
                findings
                    .iter()
                    .all(|finding| finding["kind"] == "malformed_frontmatter")
            }));
        } else if matches!(format, "markdown" | "github") {
            assert!(output_text.contains("| Inspection findings | 3 |"));
        } else {
            assert!(output_text.contains("3 spec inspection findings"));
        }
    }
}

#[test]
fn issues_ignores_nested_extension_and_block_scalar_issue_keys() {
    let tmp = TempDir::new().unwrap();
    let root = setup_minimal_project(&tmp);
    let spec_path = root.join("specs/auth/auth.spec.md");
    let spec = fs::read_to_string(&spec_path).unwrap();
    let nested_fields = "\
depends_on: []
extensions:
  implements: [999]
  nested:
    tracks: WRONG_NESTED_SHAPE_SECRET
extension_sequence:
  - implements: [998]
  - tracks: WRONG_SEQUENCE_SHAPE_SECRET
notes: |
  implements: [997]
  tracks: WRONG_BLOCK_SCALAR_SECRET";
    fs::write(spec_path, spec.replace("depends_on: []", nested_fields)).unwrap();

    let output = specsync()
        .arg("issues")
        .arg("--root")
        .arg(&root)
        .args(["--format", "json"])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    let output_text = String::from_utf8(output.clone()).unwrap();
    let json: serde_json::Value = serde_json::from_slice(&output).unwrap();

    assert_eq!(json["inspection_findings"], 0);
    assert_eq!(json["valid"], 0);
    assert_eq!(json["specs"].as_array().map(Vec::len), Some(0));
    assert!(!output_text.contains("WRONG_NESTED_SHAPE_SECRET"));
    assert!(!output_text.contains("WRONG_SEQUENCE_SHAPE_SECRET"));
    assert!(!output_text.contains("WRONG_BLOCK_SCALAR_SECRET"));
}

#[test]
fn issues_accepts_yaml_trailing_commas_and_comments_through_shared_parser() {
    let tmp = TempDir::new().unwrap();
    let root = setup_minimal_project(&tmp);
    let config_path = root.join("specsync.json");
    let config = fs::read_to_string(&config_path).unwrap();
    fs::write(
        config_path,
        config.replace(
            "\n}",
            ",\n  \"github\": { \"repo\": \"CorvidLabs/spec-sync\" }\n}",
        ),
    )
    .unwrap();
    let spec_path = root.join("specs/auth/auth.spec.md");
    let spec = fs::read_to_string(&spec_path).unwrap();
    fs::write(
        spec_path,
        spec.replace(
            "depends_on: []",
            "depends_on: []\nimplements: [42, 57,] # accepted YAML",
        ),
    )
    .unwrap();

    let output = specsync()
        .arg("issues")
        .arg("--root")
        .arg(&root)
        .args(["--format", "json"])
        .env_remove("GITHUB_TOKEN")
        .assert()
        .failure()
        .get_output()
        .stdout
        .clone();
    let json: serde_json::Value = serde_json::from_slice(&output).unwrap();

    assert_eq!(json["inspection_findings"], 0);
    assert_eq!(json["specs"].as_array().map(Vec::len), Some(1));
    assert!(
        json["errors"].as_u64().is_some_and(|errors| errors > 0),
        "valid YAML references must reach provider verification: {json}"
    );
}

#[test]
fn issues_fails_closed_on_blank_null_and_malformed_extension_yaml() {
    let tmp = TempDir::new().unwrap();
    let root = setup_minimal_project(&tmp);
    let cases = [
        ("auth", "implements: # blank is not an empty list"),
        ("billing", "tracks: null"),
        ("reporting", "extensions: [MALFORMED_EXTENSION_SECRET"),
    ];
    for (module, field) in cases {
        let spec_dir = root.join("specs").join(module);
        fs::create_dir_all(&spec_dir).unwrap();
        let spec = valid_spec(module, &["src/auth/service.ts"])
            .replace("depends_on: []", &format!("depends_on: []\n{field}"));
        fs::write(spec_dir.join(format!("{module}.spec.md")), spec).unwrap();
    }

    let output = specsync()
        .arg("issues")
        .arg("--root")
        .arg(&root)
        .args(["--format", "json"])
        .assert()
        .failure()
        .get_output()
        .stdout
        .clone();
    let output_text = String::from_utf8(output.clone()).unwrap();
    let json: serde_json::Value = serde_json::from_slice(&output).unwrap();

    assert_eq!(json["inspection_findings"], 3);
    assert!(json["findings"].as_array().is_some_and(|findings| {
        findings
            .iter()
            .all(|finding| finding["kind"] == "malformed_frontmatter")
    }));
    assert!(!output_text.contains("MALFORMED_EXTENSION_SECRET"));
    assert!(!output_text.contains("No issue references found in spec frontmatter."));
}

#[test]
fn issues_retains_spec_shaped_discovery_failures() {
    let tmp = TempDir::new().unwrap();
    let root = setup_minimal_project(&tmp);
    let broken_entry = root.join("specs/broken/broken.spec.md");
    fs::create_dir_all(&broken_entry).unwrap();

    let output = specsync()
        .arg("issues")
        .arg("--root")
        .arg(&root)
        .args(["--format", "json"])
        .assert()
        .failure()
        .get_output()
        .stdout
        .clone();
    let json: serde_json::Value = serde_json::from_slice(&output).unwrap();

    assert_eq!(json["inspection_findings"], 1);
    assert_eq!(json["findings"][0]["kind"], "discovery_error");
    assert_eq!(json["findings"][0]["spec"], "specs/broken/broken.spec.md");
    assert_eq!(
        json["findings"][0]["message"],
        "Unable to inspect spec path."
    );
}

#[cfg(target_os = "linux")]
#[test]
fn issues_retains_non_utf8_spec_filenames_as_redacted_discovery_findings() {
    use std::ffi::OsString;
    use std::os::unix::ffi::OsStringExt;

    let tmp = TempDir::new().unwrap();
    let root = setup_minimal_project(&tmp);
    let adversarial_dir = root.join("specs/adversarial");
    fs::create_dir_all(&adversarial_dir).unwrap();
    let path_secret = "NON_UTF8_PATH_SECRET_DO_NOT_PRINT";
    let mut filename = path_secret.as_bytes().to_vec();
    filename.extend_from_slice(b"\x1b\xff.spec.md");
    fs::write(
        adversarial_dir.join(OsString::from_vec(filename)),
        "missing frontmatter",
    )
    .unwrap();

    let output = specsync()
        .arg("issues")
        .arg("--root")
        .arg(&root)
        .args(["--format", "json"])
        .assert()
        .failure()
        .get_output()
        .stdout
        .clone();
    let output_text = String::from_utf8(output.clone()).unwrap();
    let json: serde_json::Value = serde_json::from_slice(&output).unwrap();

    assert_eq!(json["inspection_findings"], 1);
    assert_eq!(json["findings"][0]["kind"], "discovery_error");
    assert_eq!(json["findings"][0]["spec"], "<non-utf8-spec-path>");
    assert!(!output_text.contains(path_secret));
    assert!(!output_text.contains('\u{1b}'));
}

#[test]
fn issues_rejects_absolute_and_parent_escaping_specs_directories_without_reading_them() {
    let tmp = TempDir::new().unwrap();
    let project_root = tmp.path().join("project");
    let outside_dir = tmp.path().join("outside");
    fs::create_dir_all(&project_root).unwrap();
    fs::create_dir_all(&outside_dir).unwrap();
    let outside_secret = b"OUTSIDE_SPEC_SECRET_BYTES_DO_NOT_READ\xff";
    let outside_spec = outside_dir.join("outside.spec.md");
    fs::write(&outside_spec, outside_secret).unwrap();

    let configured_dirs = [
        outside_dir.to_string_lossy().to_string(),
        "../outside".to_string(),
    ];
    for configured_dir in configured_dirs {
        let config = serde_json::json!({
            "specsDir": configured_dir,
            "sourceDirs": ["src"],
        });
        fs::write(
            project_root.join("specsync.json"),
            serde_json::to_vec(&config).unwrap(),
        )
        .unwrap();

        for format in ["text", "table", "csv", "json", "markdown", "github"] {
            let output = specsync()
                .arg("issues")
                .arg("--root")
                .arg(&project_root)
                .args(["--format", format])
                .assert()
                .failure()
                .get_output()
                .stdout
                .clone();
            let output_text = String::from_utf8(output.clone()).unwrap();

            assert!(output_text.contains("<configured-specs-dir>"));
            assert!(
                output_text.contains("Configured specs directory is not confined to the project.")
            );
            assert!(!output_text.contains(&outside_dir.to_string_lossy().to_string()));
            assert!(!output_text.contains("OUTSIDE_SPEC_SECRET_BYTES_DO_NOT_READ"));

            if format == "json" {
                let json: serde_json::Value = serde_json::from_slice(&output).unwrap();
                assert_eq!(json["inspection_findings"], 1);
                assert_eq!(json["findings"][0]["kind"], "configuration_error");
                assert_eq!(json["findings"][0]["spec"], "<configured-specs-dir>");
            }
        }

        assert_eq!(fs::read(&outside_spec).unwrap(), outside_secret);
    }
}

#[cfg(unix)]
#[test]
fn issues_rejects_symlinked_specs_directory_without_reading_its_target() {
    use std::os::unix::fs::symlink;

    let tmp = TempDir::new().unwrap();
    let project_root = tmp.path().join("project");
    let outside_dir = tmp.path().join("outside");
    fs::create_dir_all(&project_root).unwrap();
    fs::create_dir_all(&outside_dir).unwrap();
    let outside_secret = b"SYMLINKED_OUTSIDE_SPEC_SECRET_DO_NOT_READ\xff";
    let outside_spec = outside_dir.join("outside.spec.md");
    fs::write(&outside_spec, outside_secret).unwrap();
    symlink(&outside_dir, project_root.join("linked-specs")).unwrap();
    let config = serde_json::json!({
        "specsDir": "linked-specs",
        "sourceDirs": ["src"],
    });
    fs::write(
        project_root.join("specsync.json"),
        serde_json::to_vec(&config).unwrap(),
    )
    .unwrap();

    let output = specsync()
        .arg("issues")
        .arg("--root")
        .arg(&project_root)
        .args(["--format", "json"])
        .assert()
        .failure()
        .get_output()
        .stdout
        .clone();
    let output_text = String::from_utf8(output.clone()).unwrap();
    let json: serde_json::Value = serde_json::from_slice(&output).unwrap();

    assert_eq!(json["inspection_findings"], 1);
    assert_eq!(json["findings"][0]["kind"], "configuration_error");
    assert_eq!(json["findings"][0]["spec"], "<configured-specs-dir>");
    assert!(!output_text.contains(&outside_dir.to_string_lossy().to_string()));
    assert!(!output_text.contains("SYMLINKED_OUTSIDE_SPEC_SECRET_DO_NOT_READ"));
    assert_eq!(fs::read(&outside_spec).unwrap(), outside_secret);
}

#[cfg(unix)]
#[test]
fn issues_rejects_discovered_spec_symlink_without_reading_target_bytes() {
    use std::os::unix::fs::symlink;

    let tmp = TempDir::new().unwrap();
    let root = setup_minimal_project(&tmp);
    let outside_spec = tmp.path().join("outside.spec.md");
    let outside_bytes = b"DISCOVERED_SYMLINK_SECRET_DO_NOT_READ\xff";
    fs::write(&outside_spec, outside_bytes).unwrap();
    let linked_dir = root.join("specs/linked");
    fs::create_dir_all(&linked_dir).unwrap();
    symlink(&outside_spec, linked_dir.join("linked.spec.md")).unwrap();

    let output = specsync()
        .arg("issues")
        .arg("--root")
        .arg(&root)
        .args(["--format", "json"])
        .assert()
        .failure()
        .get_output()
        .stdout
        .clone();
    let output_text = String::from_utf8(output.clone()).unwrap();
    let json: serde_json::Value = serde_json::from_slice(&output).unwrap();

    assert_eq!(json["inspection_findings"], 1);
    assert_eq!(json["findings"][0]["kind"], "discovery_error");
    assert_eq!(json["findings"][0]["spec"], "specs/linked/linked.spec.md");
    assert!(!output_text.contains("DISCOVERED_SYMLINK_SECRET_DO_NOT_READ"));
    assert!(!output_text.contains(&outside_spec.to_string_lossy().to_string()));
    assert_eq!(fs::read(&outside_spec).unwrap(), outside_bytes);
}

#[cfg(windows)]
#[test]
fn issues_rejects_windows_junction_without_reading_target_bytes() {
    let tmp = TempDir::new().unwrap();
    let root = setup_minimal_project(&tmp);
    let outside = tmp.path().join("outside");
    fs::create_dir_all(&outside).unwrap();
    let outside_spec = outside.join("outside.spec.md");
    let outside_bytes = b"WINDOWS_JUNCTION_SECRET_DO_NOT_READ\xff";
    fs::write(&outside_spec, outside_bytes).unwrap();
    let junction = root.join("specs").join("escape");
    create_windows_junction(&junction, &outside)
        .unwrap_or_else(|error| panic!("failed to create Windows junction fixture: {error}"));
    assert_eq!(
        fs::canonicalize(&junction).unwrap(),
        fs::canonicalize(&outside).unwrap()
    );

    let output = specsync()
        .arg("issues")
        .arg("--root")
        .arg(&root)
        .args(["--format", "json"])
        .assert()
        .failure()
        .get_output()
        .stdout
        .clone();
    let output_text = String::from_utf8(output.clone()).unwrap();
    let json: serde_json::Value = serde_json::from_slice(&output).unwrap();

    assert_eq!(json["inspection_findings"], 1);
    assert_eq!(json["findings"][0]["kind"], "discovery_error");
    assert_eq!(json["findings"][0]["spec"], "specs/escape");
    assert!(!output_text.contains("WINDOWS_JUNCTION_SECRET_DO_NOT_READ"));
    assert!(!output_text.contains(&outside.to_string_lossy().to_string()));
    assert_eq!(fs::read(&outside_spec).unwrap(), outside_bytes);
}

#[test]
fn issues_markdown_uses_a_code_span_longer_than_filename_backticks() {
    let tmp = TempDir::new().unwrap();
    let root = setup_minimal_project(&tmp);
    let adversarial_dir = root.join("specs/adversarial");
    fs::create_dir_all(&adversarial_dir).unwrap();
    fs::write(
        adversarial_dir.join("bad``tick.spec.md"),
        "missing frontmatter",
    )
    .unwrap();

    for format in ["markdown", "github"] {
        specsync()
            .arg("issues")
            .arg("--root")
            .arg(&root)
            .args(["--format", format])
            .assert()
            .failure()
            .stdout(predicate::str::contains(
                "```specs/adversarial/bad``tick.spec.md```",
            ));
    }
}

#[cfg(unix)]
#[test]
fn issues_sanitizes_control_characters_and_table_delimiters_in_unix_paths() {
    let tmp = TempDir::new().unwrap();
    let root = setup_minimal_project(&tmp);
    let adversarial_dir = root.join("specs/adversarial");
    fs::create_dir_all(&adversarial_dir).unwrap();
    let filename = "bad``tick|line\n\u{1b}]8;;evil.example\u{7}.spec.md";
    fs::write(adversarial_dir.join(filename), "missing frontmatter").unwrap();

    let text_output = specsync()
        .arg("issues")
        .arg("--root")
        .arg(&root)
        .assert()
        .failure()
        .get_output()
        .stdout
        .clone();
    let text = String::from_utf8(text_output).unwrap();
    assert!(text.contains("bad``tick|line\\u{000A}\\u{001B}]8;;evil.example\\u{0007}.spec.md"));
    assert!(!text.contains("\u{1b}]8"));
    assert!(!text.contains("\n\u{1b}"));

    for format in ["markdown", "github"] {
        let output = specsync()
            .arg("issues")
            .arg("--root")
            .arg(&root)
            .args(["--format", format])
            .assert()
            .failure()
            .get_output()
            .stdout
            .clone();
        let output = String::from_utf8(output).unwrap();
        assert!(output.contains(
            "```specs/adversarial/bad``tick\\|line\\\\u{000A}\\\\u{001B}]8;;evil.example\\\\u{0007}.spec.md```"
        ));
        assert!(!output.contains("\u{1b}]8"));
    }
}

#[test]
fn issues_rejects_malicious_configured_repo_without_rendering_unsafe_bytes() {
    let tmp = TempDir::new().unwrap();
    let root = setup_minimal_project(&tmp);
    let malicious_repo =
        "owner/repo\n## injected|row\u{1b}]8;;evil.example\u{7}\u{202e}\u{2028}\u{2029}";
    let config_path = root.join("specsync.json");
    let mut config: serde_json::Value =
        serde_json::from_slice(&fs::read(&config_path).unwrap()).unwrap();
    config["github"] = serde_json::json!({ "repo": malicious_repo });
    fs::write(config_path, serde_json::to_vec(&config).unwrap()).unwrap();

    for format in ["text", "table", "csv", "json", "markdown", "github"] {
        let output = specsync()
            .arg("issues")
            .arg("--root")
            .arg(&root)
            .args(["--format", format])
            .assert()
            .failure()
            .get_output()
            .clone();
        let stdout = String::from_utf8(output.stdout).unwrap();
        let stderr = String::from_utf8(output.stderr).unwrap();
        assert!(stderr.contains("GitHub repository contains invalid"));
        assert!(!stdout.contains("## injected"));
        assert!(!stderr.contains("## injected"));
        for unsafe_character in ['\u{1b}', '\u{7}', '\u{202e}', '\u{2028}', '\u{2029}'] {
            assert!(!stdout.contains(unsafe_character));
            assert!(!stderr.contains(unsafe_character));
        }
    }
}

#[cfg(unix)]
#[test]
fn issues_create_runs_normal_drift_creation_for_stable_snapshots() {
    use std::os::unix::fs::PermissionsExt;

    let tmp = TempDir::new().unwrap();
    let root = setup_minimal_project(&tmp);
    let config_path = root.join("specsync.json");
    let mut config: serde_json::Value =
        serde_json::from_slice(&fs::read(&config_path).unwrap()).unwrap();
    config["github"] = serde_json::json!({ "repo": "CorvidLabs/spec-sync" });
    fs::write(config_path, serde_json::to_vec(&config).unwrap()).unwrap();

    let spec_path = root.join("specs/auth/auth.spec.md");
    let spec = fs::read_to_string(&spec_path).unwrap();
    fs::write(
        spec_path,
        spec.replace("src/auth/service.ts", "src/auth/missing.ts"),
    )
    .unwrap();

    let bin_dir = tmp.path().join("bin");
    fs::create_dir_all(&bin_dir).unwrap();
    let fake_gh = bin_dir.join("gh");
    fs::write(
        &fake_gh,
        "#!/bin/sh\nif [ \"$1\" = \"issue\" ]; then\n  echo https://github.com/CorvidLabs/spec-sync/issues/123\nfi\nexit 0\n",
    )
    .unwrap();
    let mut permissions = fs::metadata(&fake_gh).unwrap().permissions();
    permissions.set_mode(0o755);
    fs::set_permissions(&fake_gh, permissions).unwrap();
    let mut path_entries = vec![bin_dir];
    path_entries.extend(std::env::split_paths(
        &std::env::var_os("PATH").unwrap_or_default(),
    ));
    let test_path = std::env::join_paths(path_entries).unwrap();

    specsync()
        .arg("issues")
        .arg("--create")
        .arg("--root")
        .arg(&root)
        .env("PATH", test_path)
        .assert()
        .success()
        .stdout(predicate::str::contains(
            "Creating GitHub issues for 1 spec(s) with errors...",
        ))
        .stdout(predicate::str::contains(
            "Created issue #123 for specs/auth/auth.spec.md",
        ));
}
