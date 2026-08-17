use crate::helpers::*;
use assert_cmd::Command;
use predicates::prelude::*;
use std::fs;
use tempfile::TempDir;

fn specsync_process() -> std::process::Command {
    let binary = specsync().get_program().to_os_string();
    let mut command = std::process::Command::new(binary);
    command.env_remove("GITHUB_EVENT_NAME");
    command.env_remove("GITHUB_BASE_REF");
    for (key, _) in std::env::vars_os() {
        if key
            .to_string_lossy()
            .get(.."SPECSYNC_".len())
            .is_some_and(|prefix| prefix.eq_ignore_ascii_case("SPECSYNC_"))
        {
            command.env_remove(key);
        }
    }
    command
}

// ─── 1. specsync issues ────────────────────────────────────────────────

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

#[cfg(windows)]
pub(super) fn create_windows_junction(
    junction: &std::path::Path,
    target: &std::path::Path,
) -> Result<(), String> {
    run_windows_junction_script(
        "$ErrorActionPreference = 'Stop'; New-Item -ItemType Junction \
         -Path $env:SPECSYNC_TEST_JUNCTION \
         -Target $env:SPECSYNC_TEST_TARGET | Out-Null",
        junction,
        target,
    )
}

#[cfg(windows)]
fn retarget_windows_junction(
    junction: &std::path::Path,
    target: &std::path::Path,
) -> Result<(), String> {
    fs::remove_dir(junction)
        .map_err(|error| format!("failed to remove Windows junction fixture: {error}"))?;
    create_windows_junction(junction, target)
}

#[cfg(windows)]
fn run_windows_junction_script(
    script: &str,
    junction: &std::path::Path,
    target: &std::path::Path,
) -> Result<(), String> {
    let junction = junction
        .to_str()
        .ok_or_else(|| "junction fixture path must be valid Unicode".to_string())?;
    let target = target
        .to_str()
        .ok_or_else(|| "junction target path must be valid Unicode".to_string())?;
    let mut unavailable = Vec::new();
    for executable in ["powershell.exe", "pwsh.exe"] {
        let output = match std::process::Command::new(executable)
            .args([
                "-NoLogo",
                "-NoProfile",
                "-NonInteractive",
                "-Command",
                script,
            ])
            .env("SPECSYNC_TEST_JUNCTION", junction)
            .env("SPECSYNC_TEST_TARGET", target)
            .output()
        {
            Ok(output) => output,
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
                unavailable.push(executable);
                continue;
            }
            Err(error) => {
                return Err(format!(
                    "failed to launch {executable} junction fixture: {error}"
                ));
            }
        };
        if output.status.success() {
            return Ok(());
        }
        return Err(format!(
            "{executable} junction fixture exited with {:?}; stdout: {}; stderr: {}",
            output.status.code(),
            String::from_utf8_lossy(&output.stdout).trim(),
            String::from_utf8_lossy(&output.stderr).trim()
        ));
    }
    Err(format!(
        "failed to launch a PowerShell junction fixture; unavailable executables: {}",
        unavailable.join(", ")
    ))
}

// ─── 2. specsync coverage ───────────────────────────────────────────────

#[test]
fn single_github_import_fails_closed_without_a_rest_token_or_output() {
    let tmp = TempDir::new().unwrap();
    let root = setup_minimal_project(&tmp);

    specsync()
        .arg("import")
        .args(["github", "42", "--repo", "CorvidLabs/spec-sync"])
        .arg("--root")
        .arg(&root)
        .env_remove("GITHUB_TOKEN")
        .assert()
        .failure()
        .stderr(predicate::str::contains("GITHUB_TOKEN"));

    let entries = fs::read_dir(root.join("specs"))
        .unwrap()
        .collect::<Result<Vec<_>, _>>()
        .unwrap();
    assert_eq!(entries.len(), 1, "failed import must not create a spec");
    assert!(root.join("specs/auth/auth.spec.md").exists());
}

#[test]
fn batch_github_import_fails_closed_without_a_rest_token_or_output() {
    let tmp = TempDir::new().unwrap();
    let root = setup_minimal_project(&tmp);

    specsync()
        .arg("import")
        .args(["--all-issues", "--repo", "CorvidLabs/spec-sync"])
        .arg("--root")
        .arg(&root)
        .env_remove("GITHUB_TOKEN")
        .assert()
        .failure()
        .stderr(predicate::str::contains("GITHUB_TOKEN"));

    let entries = fs::read_dir(root.join("specs"))
        .unwrap()
        .collect::<Result<Vec<_>, _>>()
        .unwrap();
    assert_eq!(entries.len(), 1, "failed batch must not create specs");
    assert!(root.join("specs/auth/auth.spec.md").exists());
}

#[test]
fn malformed_gradle_is_inconclusive_for_coverage_gating_commands() {
    let tmp = TempDir::new().unwrap();
    let root = setup_minimal_project(&tmp);
    fs::write(root.join("build.gradle.kts"), "plugins {}\n").unwrap();
    fs::write(root.join("settings.gradle.kts"), "include(\":member\"\n").unwrap();

    for command in ["check", "coverage", "generate", "report", "score"] {
        let output = specsync()
            .arg(command)
            .arg("--root")
            .arg(&root)
            .args(["--format", "json"])
            .assert()
            .failure()
            .get_output()
            .stdout
            .clone();
        let json: serde_json::Value = serde_json::from_slice(&output).unwrap_or_else(|error| {
            panic!(
                "{command} must emit valid JSON for inconclusive coverage: {error}; stdout={}",
                String::from_utf8_lossy(&output)
            )
        });
        assert_eq!(
            json["inconclusive"], true,
            "unexpected {command} JSON: {json}"
        );
        assert!(
            json["error"]
                .as_str()
                .is_some_and(|message| message.contains("Gradle")),
            "unexpected {command} error: {json}"
        );
        match command {
            "coverage" => {
                assert!(json["file_coverage"].is_null());
                assert!(json["loc_coverage"].is_null());
                assert_eq!(json["files_covered"], 0);
                assert_eq!(json["files_total"], 0);
                assert_eq!(json["loc_covered"], 0);
                assert_eq!(json["loc_total"], 0);
                assert_eq!(json["modules"], serde_json::json!([]));
                assert_eq!(json["uncovered_files"], serde_json::json!([]));
            }
            "generate" => {
                assert_eq!(json["generated"], serde_json::json!([]));
                assert!(
                    !root.join("specs/member").exists(),
                    "generate must not mutate the project after inconclusive discovery"
                );
            }
            "report" => {
                assert!(json["overall_coverage_pct"].is_null());
                assert_eq!(json["files_covered"], 0);
                assert_eq!(json["files_total"], 0);
                assert_eq!(json["total_modules"], 0);
                assert_eq!(json["stale_modules"], 0);
                assert_eq!(json["incomplete_modules"], 0);
                assert_eq!(json["modules"], serde_json::json!([]));
            }
            "score" => {
                assert!(json["average_score"].is_null());
                assert!(json["grade"].is_null());
                assert_eq!(json["total_specs"], 0);
                assert_eq!(json["specs"], serde_json::json!([]));
            }
            _ => {}
        }
    }

    specsync()
        .arg("comment")
        .arg("--root")
        .arg(&root)
        .assert()
        .failure()
        .stdout(predicate::str::is_empty())
        .stderr(predicate::str::contains("Coverage inconclusive"))
        .stderr(predicate::str::contains("Gradle"));
}

#[test]
fn invalid_utf8_source_is_inconclusive_for_coverage_gating_commands() {
    let tmp = TempDir::new().unwrap();
    let root = setup_minimal_project(&tmp);
    let source = root.join("src/auth/service.ts");
    let source_bytes = b"export function login() {}\n\xff";
    fs::write(&source, source_bytes).unwrap();

    for command in ["check", "coverage", "generate", "report", "score"] {
        let output = specsync()
            .arg(command)
            .arg("--root")
            .arg(&root)
            .args(["--format", "json"])
            .assert()
            .failure()
            .get_output()
            .stdout
            .clone();
        let json: serde_json::Value = serde_json::from_slice(&output).unwrap_or_else(|error| {
            panic!(
                "{command} must emit valid JSON for invalid UTF-8 source coverage: {error}; stdout={}",
                String::from_utf8_lossy(&output)
            )
        });
        assert_eq!(json["inconclusive"], true, "{command}: {json}");
        assert!(
            json["error"]
                .as_str()
                .is_some_and(|message| message.contains("not valid UTF-8")),
            "{command}: {json}"
        );
        if command == "generate" {
            assert_eq!(json["generated"], serde_json::json!([]));
        }
        assert_eq!(
            fs::read(&source).unwrap(),
            source_bytes,
            "{command} changed invalid source bytes"
        );
    }
}

#[test]
fn oversized_source_is_inconclusive_for_coverage_cli() {
    let tmp = TempDir::new().unwrap();
    let root = setup_minimal_project(&tmp);
    let source = root.join("src/auth/service.ts");
    fs::File::create(&source)
        .unwrap()
        .set_len(8 * 1024 * 1024 + 1)
        .unwrap();

    let output = specsync()
        .arg("coverage")
        .arg("--root")
        .arg(&root)
        .args(["--format", "json"])
        .assert()
        .failure()
        .get_output()
        .stdout
        .clone();
    let json: serde_json::Value = serde_json::from_slice(&output).unwrap_or_else(|error| {
        panic!(
            "coverage must emit valid JSON for an oversized source: {error}; stdout={}",
            String::from_utf8_lossy(&output)
        )
    });
    assert_eq!(json["inconclusive"], true, "{json}");
    assert!(
        json["error"]
            .as_str()
            .is_some_and(|message| message.contains("8 MiB per-file limit")),
        "{json}"
    );
}

#[test]
fn gradle_root_escape_is_inconclusive_for_coverage_gating_commands() {
    let tmp = TempDir::new().unwrap();
    let root = setup_minimal_project(&tmp);
    let outside_source = tmp.path().join("outside/src/main/kotlin/Secret.kt");
    fs::create_dir_all(outside_source.parent().unwrap()).unwrap();
    fs::write(&outside_source, "const val SECRET = \"DO_NOT_SCAN\"\n").unwrap();
    fs::write(
        root.join("settings.gradle.kts"),
        "include(\":outside\")\nproject(\":outside\").projectDir = file(\"../outside\")\n",
    )
    .unwrap();

    for command in ["check", "coverage", "generate", "report", "score"] {
        let output = specsync()
            .arg(command)
            .arg("--root")
            .arg(&root)
            .args(["--format", "json"])
            .assert()
            .failure()
            .get_output()
            .stdout
            .clone();
        let json: serde_json::Value = serde_json::from_slice(&output).unwrap_or_else(|error| {
            panic!(
                "{command} must emit valid JSON for a rejected Gradle escape: {error}; stdout={}",
                String::from_utf8_lossy(&output)
            )
        });
        assert_eq!(
            json["inconclusive"], true,
            "unexpected {command} JSON: {json}"
        );
        assert!(
            json["error"]
                .as_str()
                .is_some_and(|message| message.contains("must remain beneath the project root")),
            "unexpected {command} error: {json}"
        );
        assert!(
            !String::from_utf8_lossy(&output).contains("DO_NOT_SCAN"),
            "{command} disclosed outside source bytes"
        );
        assert!(
            !root.join("specs/outside").exists(),
            "{command} mutated output after rejecting the Gradle escape"
        );
    }

    assert_eq!(
        fs::read_to_string(outside_source).unwrap(),
        "const val SECRET = \"DO_NOT_SCAN\"\n"
    );
}

#[test]
fn gradle_set_project_dir_escapes_are_inconclusive_for_coverage_gating_commands() {
    for (label, project_dir) in [
        ("traversal", "../outside"),
        ("drive", "C:/outside"),
        ("unc", "//server/share/outside"),
    ] {
        let tmp = TempDir::new().unwrap();
        let root = tmp.path().join("project");
        let outside_source = tmp.path().join("outside/src/main/kotlin/Secret.kt");
        setup_minimal_project_at(&root);
        fs::create_dir_all(outside_source.parent().unwrap()).unwrap();
        let outside_bytes = format!("const val SECRET = \"SET_PROJECT_DIR_{label}\"\n");
        fs::write(&outside_source, outside_bytes.as_bytes()).unwrap();
        fs::write(
            root.join("settings.gradle.kts"),
            format!(
                "include(\":outside\")\nproject(\":outside\").setProjectDir(file(\"{project_dir}\"))\n"
            ),
        )
        .unwrap();

        assert_gradle_discovery_is_inconclusive(
            &root,
            &outside_source,
            outside_bytes.as_bytes(),
            "outside",
            label,
        );
    }
}

#[test]
fn gradle_interpolated_project_dirs_are_inconclusive_for_coverage_gating_commands() {
    for (label, override_statement) in [
        (
            "assignment-unbraced",
            r#"project(":member").projectDir = file("$outside")"#,
        ),
        (
            "setter-unbraced",
            r#"project(":member").setProjectDir(file("$outside"))"#,
        ),
        (
            "assignment-braced",
            r#"project(":member").projectDir = file("${outside}")"#,
        ),
        (
            "setter-braced",
            r#"project(":member").setProjectDir(file("${outside}"))"#,
        ),
        (
            "indirect-assignment",
            "val member = project(\":member\")\nmember.projectDir = file(\"../outside\")",
        ),
        (
            "indirect-setter",
            "val member = project(\":member\")\nmember.setProjectDir(file(\"../outside\"))",
        ),
        (
            "closure-assignment",
            "project(\":member\") {\nprojectDir = file(\"../outside\")\n}",
        ),
        (
            "whitespace-setter",
            "project(\":member\") . setProjectDir(file(\"../outside\"))",
        ),
        (
            "concatenated-constructor",
            "project(\":member\").projectDir = newFile(rootDir, \"../outside\")",
        ),
        (
            "drive-relative-selector",
            "project(\"C:outside\").projectDir = file(\"modules/member\")",
        ),
        (
            "multiline-conditional-include",
            "if (enabled) {\ninclude(\":member\")\n}",
        ),
        ("triple-quoted-include", "include(\"\"\":member\"\"\")"),
    ] {
        let tmp = TempDir::new().unwrap();
        let root = tmp.path().join("project");
        let outside_source = tmp.path().join("outside/src/main/kotlin/Secret.kt");
        setup_minimal_project_at(&root);
        fs::create_dir_all(outside_source.parent().unwrap()).unwrap();
        let outside_bytes = format!("const val SECRET = \"GRADLE_INTERPOLATION_{label}\"\n");
        fs::write(&outside_source, outside_bytes.as_bytes()).unwrap();
        fs::write(
            root.join("settings.gradle.kts"),
            format!("val outside = \"../outside\"\ninclude(\":member\")\n{override_statement}\n"),
        )
        .unwrap();

        assert_gradle_discovery_is_inconclusive(
            &root,
            &outside_source,
            outside_bytes.as_bytes(),
            "member",
            label,
        );
    }
}

#[cfg(unix)]
#[test]
fn gradle_symlink_module_escape_is_inconclusive_for_coverage_gating_commands() {
    use std::os::unix::fs::symlink;

    let project_tmp = TempDir::new().unwrap();
    let outside_tmp = TempDir::new().unwrap();
    let root = setup_minimal_project(&project_tmp);
    let outside_source = outside_tmp.path().join("src/main/kotlin/Secret.kt");
    fs::create_dir_all(outside_source.parent().unwrap()).unwrap();
    let outside_bytes = b"const val SECRET = \"GRADLE_SYMLINK_ESCAPE\"\n";
    fs::write(&outside_source, outside_bytes).unwrap();
    symlink(outside_tmp.path(), root.join("linked")).unwrap();
    fs::write(root.join("settings.gradle.kts"), "include(\":linked\")\n").unwrap();

    assert_gradle_discovery_is_inconclusive(
        &root,
        &outside_source,
        outside_bytes,
        "linked",
        "symlink",
    );
}

// Drives the product's coverage-snapshot rendezvous (`coverage_snapshot_test_barrier`),
// which is `#[cfg(debug_assertions)]`: a release build never publishes the marker, so the
// swap cannot be landed inside the window and the child exits first. Only the rendezvous
// is debug-only — the guard it synchronises (`verify_coverage_project_root`, validator.rs)
// is compiled unconditionally and is present in the shipped binary.
//
// That guard has NO release-runnable test. It is reached only from validator.rs:275 and
// :5440, and no unit test's path covers either call site. Do not mistake
// `validator::tests::retained_coverage_snapshot_rejects_post_discovery_symlink_replacement`
// for substitute coverage: it asserts "symlink or reparse point", which is the
// source-directory symlink refusal, a different guard. Nor
// `retained_coverage_sources_reject_regular_directory_replacement_after_selection`: it does
// assert "changed during retained traversal", but reaches it through
// `snapshot_selected_coverage_sources_with_hook`, so the string comes from the
// source-directory identity checks, never from `verify_coverage_project_root`.
#[cfg(unix)]
#[test]
#[cfg_attr(
    not(debug_assertions),
    ignore = "coverage-snapshot rendezvous is #[cfg(debug_assertions)]"
)]
fn gradle_post_discovery_symlink_swap_is_inconclusive_for_every_coverage_gate() {
    use std::os::unix::fs::symlink;
    use std::process::Stdio;
    use std::thread;
    use std::time::{Duration, Instant};

    const BARRIER_ENV: &str = "SPECSYNC_TEST_COVERAGE_SNAPSHOT_IDENTITY_BARRIER";
    const BARRIER_PHASE_ENV: &str = "SPECSYNC_TEST_COVERAGE_SNAPSHOT_IDENTITY_BARRIER_PHASE";
    const TEST_CONTEXT_ENV: &str = "SPECSYNC_TEST_CONTEXT";
    const TEST_CONTEXT: &str = "coverage-snapshot-identity";
    for phase in ["root-retained", "manifest-discovered"] {
        for command_name in ["check", "coverage", "generate", "report", "score"] {
            let tmp = TempDir::new().unwrap();
            let root = tmp.path().join("project");
            let original_root = tmp.path().join("original-project");
            let outside = tmp.path().join("outside");
            let barrier = tmp.path().join("barrier");
            fs::create_dir_all(root.join("member/src/main/kotlin")).unwrap();
            fs::create_dir_all(outside.join("src/main/kotlin")).unwrap();
            fs::create_dir_all(&barrier).unwrap();
            fs::write(
                root.join("member/src/main/kotlin/Local.kt"),
                "const val LOCAL = 1\n",
            )
            .unwrap();
            fs::write(root.join("settings.gradle.kts"), "include(\":member\")\n").unwrap();
            let outside_source = outside.join("src/main/kotlin/Secret.kt");
            let outside_bytes = b"const val SECRET = \"POST_DISCOVERY_SWAP\"\n";
            fs::write(&outside_source, outside_bytes).unwrap();

            let mut process = specsync_process();
            process
                .arg(command_name)
                .arg("--root")
                .arg(&root)
                .args(["--format", "json"]);
            if command_name == "score" {
                process.args(["--require-coverage", "100"]);
            }
            let mut child = process
                .env(BARRIER_ENV, &barrier)
                .env(BARRIER_PHASE_ENV, phase)
                .env(TEST_CONTEXT_ENV, TEST_CONTEXT)
                .stdin(Stdio::null())
                .stdout(Stdio::piped())
                .stderr(Stdio::piped())
                .spawn()
                .unwrap();

            let ready = barrier.join(phase);
            let deadline = Instant::now() + Duration::from_secs(10);
            while !ready.is_file() {
                assert!(
                    child.try_wait().unwrap().is_none(),
                    "{command_name} exited before the {phase} coverage barrier"
                );
                assert!(
                    Instant::now() < deadline,
                    "{command_name} did not reach the {phase} coverage barrier"
                );
                thread::sleep(Duration::from_millis(10));
            }

            fs::rename(&root, &original_root).unwrap();
            symlink(&outside, &root).unwrap();
            fs::write(barrier.join("resume"), b"resume\n").unwrap();
            let output = child.wait_with_output().unwrap();

            assert!(
                !output.status.success(),
                "{command_name} false green at {phase}"
            );
            let json: serde_json::Value =
                serde_json::from_slice(&output.stdout).unwrap_or_else(|error| {
                    panic!(
                        "{command_name} must emit JSON after a {phase} swap: {error}; stdout={}",
                        String::from_utf8_lossy(&output.stdout)
                    )
                });
            assert_eq!(json["inconclusive"], true, "{command_name}: {json}");
            assert!(
                json["error"]
                    .as_str()
                    .is_some_and(|error| error.contains("project root")
                        && error.contains("changed during retained traversal")),
                "{command_name}: {json}"
            );
            assert!(!String::from_utf8_lossy(&output.stdout).contains("POST_DISCOVERY_SWAP"));
            assert!(!root.join("specs/member").exists());
            assert!(!original_root.join("specs/member").exists());
            assert_eq!(fs::read(&outside_source).unwrap(), outside_bytes);
        }
    }
}

#[cfg(any(unix, windows))]
fn assert_generate_post_coverage_root_retarget_is_inconclusive<CreateAlias, RetargetAlias>(
    create_alias: CreateAlias,
    retarget_alias: RetargetAlias,
) where
    CreateAlias: Fn(&std::path::Path, &std::path::Path) -> Result<(), String>,
    RetargetAlias: Fn(&std::path::Path, &std::path::Path) -> Result<(), String>,
{
    use std::process::Stdio;
    use std::thread;
    use std::time::{Duration, Instant};

    const BARRIER_ENV: &str = "SPECSYNC_TEST_GENERATE_ROOT_IDENTITY_BARRIER";
    const TEST_CONTEXT_ENV: &str = "SPECSYNC_TEST_CONTEXT";
    const TEST_CONTEXT: &str = "generate-root-identity";
    const MARKER: &str = "coverage-complete";

    const MODULES: [(&str, &[u8], &[u8]); 2] = [
        (
            "member",
            b"pub fn retained_member() {}\n",
            b"pub fn replacement_member() {}\n",
        ),
        (
            "peer",
            b"pub fn retained_peer() {}\n",
            b"pub fn replacement_peer() {}\n",
        ),
    ];

    for (format, batch) in [
        ("text", false),
        ("json", false),
        ("text", true),
        ("json", true),
    ] {
        let case = format!("{format}/{}", if batch { "batch" } else { "default" });
        let tmp = TempDir::new().unwrap();
        let root = tmp.path().join("requested-project");
        let original = tmp.path().join("original-project");
        let replacement = tmp.path().join("replacement-project");
        let barrier = tmp.path().join("barrier");
        fs::create_dir_all(&original).unwrap();
        fs::create_dir_all(&replacement).unwrap();
        fs::create_dir_all(&barrier).unwrap();
        write_config(&original, "specs", &["src"]);
        write_config(&replacement, "specs", &["src"]);
        let original_config = original.join("specsync.json");
        let original_config_bytes = fs::read(&original_config).unwrap();
        let replacement_config = replacement.join("specsync.json");
        let replacement_config_bytes = fs::read(&replacement_config).unwrap();
        for &(module, original_source_bytes, replacement_source_bytes) in &MODULES {
            let original_source = original.join("src").join(module).join("local.rs");
            let replacement_source = replacement.join("src").join(module).join("replacement.rs");
            fs::create_dir_all(original_source.parent().unwrap()).unwrap();
            fs::create_dir_all(replacement_source.parent().unwrap()).unwrap();
            fs::write(original_source, original_source_bytes).unwrap();
            fs::write(replacement_source, replacement_source_bytes).unwrap();
        }
        let original_sentinel = original.join("original-sentinel.bin");
        let original_sentinel_bytes = b"retained project must remain byte exact\0";
        fs::write(&original_sentinel, original_sentinel_bytes).unwrap();
        let replacement_sentinel = replacement.join("replacement-sentinel.bin");
        let replacement_sentinel_bytes = b"replacement project must remain byte exact\0";
        fs::write(&replacement_sentinel, replacement_sentinel_bytes).unwrap();
        create_alias(&root, &original)
            .unwrap_or_else(|error| panic!("failed to create requested-root alias: {error}"));

        let mut process = specsync_process();
        process
            .arg("generate")
            .arg("--root")
            .arg(&root)
            .args(["--format", format]);
        if batch {
            process.args(["--batch", "member,peer"]);
        }
        let mut child = process
            .env(BARRIER_ENV, &barrier)
            .env(TEST_CONTEXT_ENV, TEST_CONTEXT)
            .stdin(Stdio::null())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .unwrap();

        let ready = barrier.join(MARKER);
        let deadline = Instant::now() + Duration::from_secs(10);
        while !ready.is_file() {
            assert!(
                child.try_wait().unwrap().is_none(),
                "generate exited before the post-coverage barrier; case={case}"
            );
            assert!(
                Instant::now() < deadline,
                "generate did not reach the post-coverage barrier; case={case}"
            );
            thread::sleep(Duration::from_millis(10));
        }

        retarget_alias(&root, &replacement)
            .unwrap_or_else(|error| panic!("failed to retarget requested-root alias: {error}"));
        fs::write(barrier.join("resume"), b"resume\n").unwrap();
        let output = child.wait_with_output().unwrap();

        assert!(
            !output.status.success(),
            "generate false green after a post-coverage root retarget; case={case}"
        );
        if format == "json" {
            let json: serde_json::Value =
                serde_json::from_slice(&output.stdout).unwrap_or_else(|error| {
                    panic!(
                        "generate must emit inconclusive JSON after root retarget: {error}; case={case}; stdout={}",
                        String::from_utf8_lossy(&output.stdout)
                    )
                });
            assert_eq!(json["inconclusive"], true, "case={case}: {json}");
            assert_eq!(
                json["generated"],
                serde_json::json!([]),
                "case={case}: {json}"
            );
            assert!(
                json["error"]
                    .as_str()
                    .is_some_and(|error| error.contains("Generate project root")
                        && error.contains("changed after coverage")),
                "case={case}: {json}"
            );
        } else {
            let stderr = String::from_utf8_lossy(&output.stderr);
            assert!(
                stderr.contains("Generation inconclusive:")
                    && stderr.contains("Generate project root")
                    && stderr.contains("changed after coverage"),
                "text generation must report an inconclusive root retarget; case={case}; stderr={stderr}"
            );
            let stdout = String::from_utf8_lossy(&output.stdout);
            assert!(
                !stdout.contains("Batch generate complete:") && !stdout.contains("\n  Generated "),
                "text generation falsely reported successful output; case={case}; stdout={stdout}"
            );
        }

        assert_eq!(
            fs::read(&original_config).unwrap(),
            original_config_bytes,
            "generation changed retained config bytes; case={case}"
        );
        assert_eq!(
            fs::read(&replacement_config).unwrap(),
            replacement_config_bytes,
            "generation changed replacement config bytes; case={case}"
        );
        assert_eq!(
            fs::read(&original_sentinel).unwrap(),
            original_sentinel_bytes,
            "generation changed retained sentinel bytes; case={case}"
        );
        assert_eq!(
            fs::read(&replacement_sentinel).unwrap(),
            replacement_sentinel_bytes,
            "generation changed replacement sentinel bytes; case={case}"
        );
        for &(module, original_source_bytes, replacement_source_bytes) in &MODULES {
            assert!(
                !original.join("specs").join(module).exists(),
                "failed generation changed the retained output tree; case={case}; module={module}"
            );
            assert!(
                !replacement.join("specs").join(module).exists(),
                "generation wrote through the replacement root; case={case}; module={module}"
            );
            assert_eq!(
                fs::read(original.join("src").join(module).join("local.rs")).unwrap(),
                original_source_bytes,
                "generation changed retained source bytes; case={case}; module={module}"
            );
            assert_eq!(
                fs::read(replacement.join("src").join(module).join("replacement.rs")).unwrap(),
                replacement_source_bytes,
                "generation changed replacement source bytes; case={case}; module={module}"
            );
        }
    }
}

// Drives the product's post-coverage rendezvous (`generate_after_coverage_test_barrier`),
// which is `#[cfg(debug_assertions)]`. Only the rendezvous is debug-only — the guard it
// synchronises (`RetainedGenerateRoot::verify_public_path`) is compiled unconditionally
// and is present in the shipped binary.
//
// That guard has no release-runnable test, and no unit coverage in ANY profile:
// src/commands/generate.rs has no `mod tests` at all. This test and its Windows sibling
// are its only coverage, so in release it has none.
#[cfg(unix)]
#[test]
#[cfg_attr(
    not(debug_assertions),
    ignore = "generate post-coverage rendezvous is #[cfg(debug_assertions)]"
)]
fn generate_post_coverage_symlink_root_retarget_is_inconclusive_before_writes() {
    use std::os::unix::fs::symlink;

    assert_generate_post_coverage_root_retarget_is_inconclusive(
        |alias, target| symlink(target, alias).map_err(|error| error.to_string()),
        |alias, target| {
            fs::remove_file(alias).map_err(|error| error.to_string())?;
            symlink(target, alias).map_err(|error| error.to_string())
        },
    );
}

// Same debug-only rendezvous as the symlink variant above, so it carries the same gate.
// This test does run on Windows during RC qualification (release.yml `qualify` →
// `fledge lanes run release-candidate` → `cargo test`, a debug build), where the gate is a
// no-op. It would fail only under `cargo test --release` on Windows, which nothing runs.
#[cfg(windows)]
#[test]
#[cfg_attr(
    not(debug_assertions),
    ignore = "generate post-coverage rendezvous is #[cfg(debug_assertions)]"
)]
fn generate_post_coverage_junction_root_retarget_is_inconclusive_before_writes() {
    assert_generate_post_coverage_root_retarget_is_inconclusive(
        create_windows_junction,
        retarget_windows_junction,
    );
}

#[cfg(unix)]
#[test]
fn gradle_symlinked_manifests_are_inconclusive_without_reading_outside_bytes() {
    use std::os::unix::fs::symlink;

    for manifest_name in ["build.gradle.kts", "settings.gradle.kts"] {
        let project_tmp = TempDir::new().unwrap();
        let outside_tmp = TempDir::new().unwrap();
        let root = setup_minimal_project(&project_tmp);
        let outside_manifest = outside_tmp.path().join(manifest_name);
        let outside_bytes = b"include(\":GRADLE_MANIFEST_SECRET\")\n";
        fs::write(&outside_manifest, outside_bytes).unwrap();
        symlink(&outside_manifest, root.join(manifest_name)).unwrap();

        for command in ["check", "coverage", "generate", "report", "score"] {
            let output = specsync()
                .arg(command)
                .arg("--root")
                .arg(&root)
                .args(["--format", "json"])
                .assert()
                .failure()
                .get_output()
                .stdout
                .clone();
            let json: serde_json::Value =
                serde_json::from_slice(&output).unwrap_or_else(|error| {
                    panic!(
                        "{command} must emit valid JSON for a linked Gradle manifest: {error}; stdout={}",
                        String::from_utf8_lossy(&output)
                    )
                });
            assert_eq!(
                json["inconclusive"], true,
                "unexpected {command} JSON for {manifest_name}: {json}"
            );
            assert!(
                json["error"]
                    .as_str()
                    .is_some_and(|message| message.contains("symlink or reparse point")),
                "unexpected {command} error for {manifest_name}: {json}"
            );
            assert!(
                !String::from_utf8_lossy(&output).contains("GRADLE_MANIFEST_SECRET"),
                "{command} disclosed outside Gradle manifest bytes"
            );
            assert!(
                !root.join("specs/GRADLE_MANIFEST_SECRET").exists(),
                "{command} mutated output after rejecting {manifest_name}"
            );
            assert_eq!(
                fs::read(&outside_manifest).unwrap(),
                outside_bytes,
                "{command} changed outside Gradle manifest bytes"
            );
        }
    }
}

#[cfg(windows)]
#[test]
fn gradle_junction_module_escape_is_inconclusive_for_coverage_gating_commands() {
    let project_tmp = TempDir::new().unwrap();
    let outside_tmp = TempDir::new().unwrap();
    let root = setup_minimal_project(&project_tmp);
    let outside_source = outside_tmp.path().join("src/main/kotlin/Secret.kt");
    fs::create_dir_all(outside_source.parent().unwrap()).unwrap();
    let outside_bytes = b"const val SECRET = \"GRADLE_JUNCTION_ESCAPE\"\n";
    fs::write(&outside_source, outside_bytes).unwrap();
    create_windows_junction(&root.join("linked"), outside_tmp.path())
        .unwrap_or_else(|error| panic!("failed to create Gradle module junction fixture: {error}"));
    fs::write(root.join("settings.gradle.kts"), "include(\":linked\")\n").unwrap();

    assert_gradle_discovery_is_inconclusive(
        &root,
        &outside_source,
        outside_bytes,
        "linked",
        "junction",
    );
}

// Same debug-only coverage-snapshot rendezvous as the symlink variant above, so it carries
// the same gate. Like the junction variant above, it does run on Windows during RC
// qualification (a debug build), where the gate is a no-op.
#[cfg(windows)]
#[test]
#[cfg_attr(
    not(debug_assertions),
    ignore = "coverage-snapshot rendezvous is #[cfg(debug_assertions)]"
)]
fn gradle_post_discovery_junction_swap_is_inconclusive_for_every_coverage_gate() {
    use std::process::Stdio;
    use std::thread;
    use std::time::{Duration, Instant};

    const BARRIER_ENV: &str = "SPECSYNC_TEST_COVERAGE_SNAPSHOT_IDENTITY_BARRIER";
    const BARRIER_PHASE_ENV: &str = "SPECSYNC_TEST_COVERAGE_SNAPSHOT_IDENTITY_BARRIER_PHASE";
    const TEST_CONTEXT_ENV: &str = "SPECSYNC_TEST_CONTEXT";
    const TEST_CONTEXT: &str = "coverage-snapshot-identity";
    for phase in ["root-retained", "manifest-discovered"] {
        for command_name in ["check", "coverage", "generate", "report", "score"] {
            let tmp = TempDir::new().unwrap();
            let root = tmp.path().join("project");
            let original_root = tmp.path().join("original-project");
            let outside = tmp.path().join("outside");
            let barrier = tmp.path().join("barrier");
            fs::create_dir_all(original_root.join("member/src/main/kotlin")).unwrap();
            fs::create_dir_all(outside.join("src/main/kotlin")).unwrap();
            fs::create_dir_all(&barrier).unwrap();
            fs::write(
                original_root.join("member/src/main/kotlin/Local.kt"),
                "const val LOCAL = 1\n",
            )
            .unwrap();
            fs::write(
                original_root.join("settings.gradle.kts"),
                "include(\":member\")\n",
            )
            .unwrap();
            let outside_source = outside.join("src/main/kotlin/Secret.kt");
            let outside_bytes = b"const val SECRET = \"POST_DISCOVERY_JUNCTION_SWAP\"\n";
            fs::write(&outside_source, outside_bytes).unwrap();
            create_windows_junction(&root, &original_root).unwrap_or_else(|error| {
                panic!("failed to create initial project-root junction fixture: {error}")
            });
            assert_eq!(
                fs::canonicalize(&root).unwrap(),
                fs::canonicalize(&original_root).unwrap(),
                "initial project-root junction does not target the local project"
            );

            let mut process = specsync_process();
            process
                .arg(command_name)
                .arg("--root")
                .arg(&root)
                .args(["--format", "json"]);
            if command_name == "score" {
                process.args(["--require-coverage", "100"]);
            }
            let mut child = process
                .env(BARRIER_ENV, &barrier)
                .env(BARRIER_PHASE_ENV, phase)
                .env(TEST_CONTEXT_ENV, TEST_CONTEXT)
                .stdin(Stdio::null())
                .stdout(Stdio::piped())
                .stderr(Stdio::piped())
                .spawn()
                .unwrap();

            let ready = barrier.join(phase);
            let deadline = Instant::now() + Duration::from_secs(10);
            while !ready.is_file() {
                assert!(
                    child.try_wait().unwrap().is_none(),
                    "{command_name} exited before the {phase} coverage barrier"
                );
                assert!(
                    Instant::now() < deadline,
                    "{command_name} did not reach the {phase} coverage barrier"
                );
                thread::sleep(Duration::from_millis(10));
            }

            retarget_windows_junction(&root, &outside).unwrap_or_else(|error| {
                panic!("failed to retarget post-discovery project-root junction fixture: {error}")
            });
            assert_eq!(
                fs::canonicalize(&root).unwrap(),
                fs::canonicalize(&outside).unwrap(),
                "replacement project-root junction does not target the outside directory"
            );
            fs::write(barrier.join("resume"), b"resume\n").unwrap();
            let output = child.wait_with_output().unwrap();

            assert!(
                !output.status.success(),
                "{command_name} false green at {phase}"
            );
            let json: serde_json::Value =
            serde_json::from_slice(&output.stdout).unwrap_or_else(|error| {
                panic!(
                    "{command_name} must emit JSON after a {phase} junction swap: {error}; stdout={}",
                    String::from_utf8_lossy(&output.stdout)
                )
            });
            assert_eq!(json["inconclusive"], true, "{command_name}: {json}");
            assert!(
                json["error"]
                    .as_str()
                    .is_some_and(|error| error.contains("project root")
                        && error.contains("changed during retained traversal")),
                "{command_name}: {json}"
            );
            assert!(
                !String::from_utf8_lossy(&output.stdout).contains("POST_DISCOVERY_JUNCTION_SWAP")
            );
            assert!(!root.join("specs/member").exists());
            assert!(!original_root.join("specs/member").exists());
            assert_eq!(fs::read(&outside_source).unwrap(), outside_bytes);
        }
    }
}

fn setup_minimal_project_at(root: &std::path::Path) {
    fs::create_dir_all(root.join("src/auth")).unwrap();
    fs::create_dir_all(root.join("specs/auth")).unwrap();
    write_config(root, "specs", &["src"]);
    fs::write(
        root.join("src/auth/service.ts"),
        "export function login() {}\nexport function logout() {}\n",
    )
    .unwrap();
    fs::write(
        root.join("specs/auth/auth.spec.md"),
        valid_spec("auth", &["src/auth/service.ts"]),
    )
    .unwrap();
}

fn assert_gradle_discovery_is_inconclusive(
    root: &std::path::Path,
    outside_source: &std::path::Path,
    outside_bytes: &[u8],
    module: &str,
    label: &str,
) {
    for command in ["check", "coverage", "generate", "report", "score"] {
        let output = specsync()
            .arg(command)
            .arg("--root")
            .arg(root)
            .args(["--format", "json"])
            .assert()
            .failure()
            .get_output()
            .stdout
            .clone();
        let json: serde_json::Value = serde_json::from_slice(&output).unwrap_or_else(|error| {
            panic!(
                "{command} must emit valid JSON for rejected Gradle {label} discovery: {error}; stdout={}",
                String::from_utf8_lossy(&output)
            )
        });
        assert_eq!(
            json["inconclusive"], true,
            "unexpected {command} JSON for Gradle {label} discovery: {json}"
        );
        assert!(
            json["error"]
                .as_str()
                .is_some_and(|message| message.contains("Gradle")),
            "unexpected {command} error for Gradle {label} discovery: {json}"
        );
        assert!(
            !String::from_utf8_lossy(&output).contains("SECRET"),
            "{command} disclosed outside source bytes for Gradle {label} discovery"
        );
        assert!(
            !root.join("specs").join(module).exists(),
            "{command} mutated output after rejecting Gradle {label} discovery"
        );
        assert_eq!(
            fs::read(outside_source).unwrap(),
            outside_bytes,
            "{command} changed outside bytes for Gradle {label} discovery"
        );
    }
}

#[test]
fn coverage_full_reports_100() {
    let tmp = TempDir::new().unwrap();
    let root = setup_minimal_project(&tmp);

    specsync()
        .arg("coverage")
        .arg("--root")
        .arg(&root)
        .assert()
        .success()
        .stdout(predicate::str::contains("100%"));
}

#[test]
fn coverage_partial_lists_unspecced_files() {
    let tmp = TempDir::new().unwrap();
    let root = setup_minimal_project(&tmp);

    // Add a second source file not covered by any spec.
    fs::create_dir_all(root.join("src/auth")).unwrap();
    fs::write(
        root.join("src/auth/middleware.ts"),
        "export function protect() {}\n",
    )
    .unwrap();

    specsync()
        .arg("coverage")
        .arg("--root")
        .arg(&root)
        .assert()
        .success()
        .stdout(predicate::str::contains("src/auth/middleware.ts"));
}

#[test]
fn coverage_shows_unspecced_modules() {
    let tmp = TempDir::new().unwrap();
    let root = setup_minimal_project(&tmp);

    // Add a new module directory with no corresponding spec dir.
    fs::create_dir_all(root.join("src/billing")).unwrap();
    fs::write(
        root.join("src/billing/invoice.ts"),
        "export function createInvoice() {}\n",
    )
    .unwrap();

    specsync()
        .arg("coverage")
        .arg("--root")
        .arg(&root)
        .assert()
        .success()
        .stdout(predicate::str::contains("billing"));
}

// ─── 3. specsync generate ───────────────────────────────────────────────

#[test]
fn generate_creates_spec_for_unspecced_module() {
    let tmp = TempDir::new().unwrap();
    let root = setup_minimal_project(&tmp);

    // Add unspecced module
    fs::create_dir_all(root.join("src/payments")).unwrap();
    fs::write(
        root.join("src/payments/processor.ts"),
        "export function charge() {}\n",
    )
    .unwrap();

    specsync()
        .arg("generate")
        .arg("--root")
        .arg(&root)
        .assert()
        .success()
        .stdout(predicate::str::contains("Generated"));

    // Verify spec file was created
    let spec_path = root.join("specs/payments/payments.spec.md");
    assert!(spec_path.exists(), "Generated spec file should exist");
    let content = fs::read_to_string(&spec_path).unwrap();
    assert!(content.contains("module: payments"));
}

#[test]
fn generate_batch_creates_only_the_selected_unspecced_module() {
    let tmp = TempDir::new().unwrap();
    let root = setup_minimal_project(&tmp);
    for module in ["payments", "shipping"] {
        fs::create_dir_all(root.join("src").join(module)).unwrap();
        fs::write(
            root.join("src").join(module).join("service.ts"),
            format!("export function {module}() {{}}\n"),
        )
        .unwrap();
    }

    specsync()
        .args(["generate", "--batch", "payments", "--root"])
        .arg(&root)
        .assert()
        .success()
        .stdout(predicate::str::contains("Batch generate complete: 1/1"));

    assert!(root.join("specs/payments/payments.spec.md").is_file());
    assert!(!root.join("specs/shipping").exists());
}

#[test]
fn generate_no_op_when_fully_covered() {
    let tmp = TempDir::new().unwrap();
    let root = setup_minimal_project(&tmp);

    specsync()
        .arg("generate")
        .arg("--root")
        .arg(&root)
        .assert()
        .success()
        .stdout(predicate::str::contains("No specs to generate"));
}

#[test]
fn generate_rejects_retired_provider_and_model_flags() {
    let tmp = TempDir::new().unwrap();
    let root = setup_minimal_project(&tmp);

    specsync()
        .args(["generate", "--provider", "openai", "--root"])
        .arg(&root)
        .assert()
        .failure()
        .stderr(predicate::str::contains("unexpected argument '--provider'"));

    specsync()
        .args(["generate", "--model", "secret-model", "--root"])
        .arg(&root)
        .assert()
        .failure()
        .stderr(predicate::str::contains("unexpected argument '--model'"));
}

#[test]
fn generate_never_executes_legacy_ai_command_environment_variable() {
    let tmp = TempDir::new().unwrap();
    let root = setup_minimal_project(&tmp);
    fs::create_dir_all(root.join("src/payments")).unwrap();
    fs::write(
        root.join("src/payments/processor.ts"),
        "export function charge() {}\n",
    )
    .unwrap();
    let marker = root.join("legacy-ai-command-executed");
    let command = format!("touch {}", marker.display());

    let secret = "sk-environment-must-not-affect-generation";
    let output = specsync()
        .args(["generate", "--root"])
        .arg(&root)
        .env("SPECSYNC_AI_COMMAND", command)
        .env("SPECSYNC_AI_PROVIDER", "anthropic")
        .env("SPECSYNC_AI_MODEL", "retired-model")
        .env("ANTHROPIC_API_KEY", secret)
        .output()
        .unwrap();

    assert!(output.status.success());
    assert!(
        !marker.exists(),
        "retired command environment variable executed"
    );
    assert!(!String::from_utf8_lossy(&output.stdout).contains(secret));
    assert!(!String::from_utf8_lossy(&output.stderr).contains(secret));
    assert!(root.join("specs/payments/payments.spec.md").exists());
}

#[test]
fn check_fix_never_executes_legacy_ai_command_environment_variable() {
    let tmp = TempDir::new().unwrap();
    let root = setup_minimal_project(&tmp);
    fs::write(
        root.join("src/auth/service.ts"),
        "export function login() {}\nexport function logout() {}\nexport function refresh() {}\n",
    )
    .unwrap();
    let marker = root.join("legacy-check-fix-command-executed");
    let command = format!("touch {}", marker.display());

    specsync()
        .args(["check", "--fix", "--root"])
        .arg(&root)
        .env("SPECSYNC_AI_COMMAND", command)
        .assert()
        .success();

    assert!(
        !marker.exists(),
        "retired command executed during check --fix"
    );
    let spec = fs::read_to_string(root.join("specs/auth/auth.spec.md")).unwrap();
    assert!(spec.contains("`refresh`"));
}

// ─── 4. specsync init ───────────────────────────────────────────────────

#[test]
fn init_creates_config_file() {
    let tmp = TempDir::new().unwrap();
    let root = tmp.path();

    specsync()
        .arg("init")
        .arg("--root")
        .arg(root)
        .assert()
        .success()
        .stdout(predicate::str::contains("Created .specsync/config.toml"));

    let config_path = root.join(".specsync/config.toml");
    assert!(config_path.exists());
    let content = fs::read_to_string(&config_path).unwrap();
    assert!(content.contains("specs_dir"));
    assert!(content.contains("source_dirs"));
    assert!(content.contains("required_sections"));

    // Full v4 layout — version stamp, .gitignore, and state directories
    assert!(root.join(".specsync/version").exists());
    assert!(root.join(".specsync/.gitignore").exists());
    assert!(root.join(".specsync/lifecycle").is_dir());
    assert!(root.join(".specsync/changes").is_dir());
    assert!(root.join(".specsync/archive").is_dir());
}

#[test]
fn init_then_check_is_usable_without_git_and_does_not_nag_about_legacy_layout() {
    let tmp = TempDir::new().unwrap();
    let root = tmp.path();

    fs::create_dir_all(root.join("src")).unwrap();
    fs::write(root.join("src/main.rs"), "fn main() {}").unwrap();

    specsync()
        .arg("init")
        .arg("--root")
        .arg(root)
        .assert()
        .success();

    let policy: serde_json::Value =
        serde_json::from_str(&fs::read_to_string(root.join(".specsync/sdd.json")).unwrap())
            .unwrap();
    assert_eq!(policy["enabled"], true);
    assert_eq!(policy["require_change_for_meaningful_files"], false);

    // Lifecycle checks remain available without requiring impossible Git diff evidence.
    specsync()
        .arg("check")
        .arg("--root")
        .arg(root)
        .assert()
        .success()
        .stderr(predicate::str::contains("Legacy 3.x layout").not());
}

#[test]
fn init_does_not_overwrite_existing_v4_config() {
    let tmp = TempDir::new().unwrap();
    let root = tmp.path();

    fs::create_dir_all(root.join(".specsync")).unwrap();
    fs::write(
        root.join(".specsync/config.toml"),
        "specs_dir = \"custom\"\n",
    )
    .unwrap();

    specsync()
        .arg("init")
        .arg("--root")
        .arg(root)
        .assert()
        .success()
        .stdout(predicate::str::contains("already exists"));

    let content = fs::read_to_string(root.join(".specsync/config.toml")).unwrap();
    assert!(content.contains("custom"));
}

#[test]
fn init_does_not_overwrite_existing_config() {
    let tmp = TempDir::new().unwrap();
    let root = tmp.path();

    fs::write(root.join("specsync.json"), r#"{"specsDir":"custom"}"#).unwrap();

    specsync()
        .arg("init")
        .arg("--root")
        .arg(root)
        .assert()
        .success()
        .stdout(predicate::str::contains("already exists"));

    // Original content preserved
    let content = fs::read_to_string(root.join("specsync.json")).unwrap();
    assert!(content.contains("custom"));
}

// ─── Auto-detect source directories ─────────────────────────────────────

#[test]
fn init_auto_detects_src_dir() {
    let tmp = TempDir::new().unwrap();
    let root = tmp.path();

    // Create a project with src/ containing source files
    fs::create_dir_all(root.join("src")).unwrap();
    fs::write(root.join("src/main.rs"), "fn main() {}").unwrap();

    specsync()
        .arg("init")
        .arg("--root")
        .arg(root)
        .assert()
        .success()
        .stdout(predicate::str::contains("Detected source directories: src"));

    let config = fs::read_to_string(root.join(".specsync/config.toml")).unwrap();
    assert!(config.contains("source_dirs = [\"src\"]"));
}

#[test]
fn init_auto_detects_lib_dir() {
    let tmp = TempDir::new().unwrap();
    let root = tmp.path();

    // Create a project with lib/ containing source files
    fs::create_dir_all(root.join("lib")).unwrap();
    fs::write(root.join("lib/utils.py"), "def hello(): pass\n").unwrap();

    specsync()
        .arg("init")
        .arg("--root")
        .arg(root)
        .assert()
        .success()
        .stdout(predicate::str::contains("Detected source directories: lib"));

    let config = fs::read_to_string(root.join(".specsync/config.toml")).unwrap();
    assert!(config.contains("source_dirs = [\"lib\"]"));
}

#[test]
fn init_auto_detects_multiple_dirs() {
    let tmp = TempDir::new().unwrap();
    let root = tmp.path();

    // Create a project with both src/ and lib/
    fs::create_dir_all(root.join("src")).unwrap();
    fs::write(root.join("src/main.ts"), "export function main() {}").unwrap();
    fs::create_dir_all(root.join("lib")).unwrap();
    fs::write(root.join("lib/helpers.ts"), "export function help() {}").unwrap();

    specsync()
        .arg("init")
        .arg("--root")
        .arg(root)
        .assert()
        .success()
        .stdout(predicate::str::contains(
            "Detected source directories: lib, src",
        ));

    let config = fs::read_to_string(root.join(".specsync/config.toml")).unwrap();
    assert!(config.contains("source_dirs = [\"lib\", \"src\"]"));
}

#[test]
fn init_ignores_node_modules_and_hidden_dirs() {
    let tmp = TempDir::new().unwrap();
    let root = tmp.path();

    // Create source in app/ and noise in node_modules/ and .cache/
    fs::create_dir_all(root.join("app")).unwrap();
    fs::write(root.join("app/index.ts"), "export default function() {}").unwrap();
    fs::create_dir_all(root.join("node_modules/some-pkg")).unwrap();
    fs::write(
        root.join("node_modules/some-pkg/index.js"),
        "module.exports = {}",
    )
    .unwrap();
    fs::create_dir_all(root.join(".cache")).unwrap();
    fs::write(root.join(".cache/data.js"), "const x = 1;").unwrap();

    specsync()
        .arg("init")
        .arg("--root")
        .arg(root)
        .assert()
        .success()
        .stdout(predicate::str::contains("Detected source directories: app"));

    let config = fs::read_to_string(root.join(".specsync/config.toml")).unwrap();
    assert!(config.contains("source_dirs = [\"app\"]"));
}

#[test]
fn check_works_without_config_file() {
    let tmp = TempDir::new().unwrap();
    let root = tmp.path();

    // Create a project with lib/ source and specs, but no specsync.json
    fs::create_dir_all(root.join("lib/auth")).unwrap();
    fs::write(
        root.join("lib/auth/service.ts"),
        "export function login() {}\nexport function logout() {}\n",
    )
    .unwrap();
    fs::create_dir_all(root.join("specs/auth")).unwrap();
    let spec = valid_spec("auth", &["lib/auth/service.ts"]);
    fs::write(root.join("specs/auth/auth.spec.md"), spec).unwrap();

    // Should auto-detect lib/ and work without any config
    specsync()
        .arg("check")
        .arg("--root")
        .arg(root)
        .assert()
        .success();
}

#[test]
fn init_falls_back_to_src_when_no_source_files() {
    let tmp = TempDir::new().unwrap();
    let root = tmp.path();

    // Empty project with only a README
    fs::write(root.join("README.md"), "# My Project").unwrap();

    specsync()
        .arg("init")
        .arg("--root")
        .arg(root)
        .assert()
        .success()
        .stdout(predicate::str::contains("Detected source directories: src"));

    let config = fs::read_to_string(root.join(".specsync/config.toml")).unwrap();
    assert!(config.contains("source_dirs = [\"src\"]"));
}

// ─── Score Command Tests ─────────────────────────────────────────────────

#[test]
fn score_command_outputs_quality_grades() {
    let tmp = TempDir::new().unwrap();
    let root = tmp.path();

    fs::create_dir_all(root.join("src")).unwrap();
    fs::write(root.join("src/auth.ts"), "export function login() {}").unwrap();

    fs::create_dir_all(root.join("specs/auth")).unwrap();
    fs::write(
        root.join("specs/auth/auth.spec.md"),
        valid_spec("auth", &["src/auth.ts"]),
    )
    .unwrap();

    specsync()
        .args(["score", "--root", root.to_str().unwrap()])
        .assert()
        .success()
        .stdout(predicate::str::contains("/100"));
}

#[test]
fn score_json_output_has_grades() {
    let tmp = TempDir::new().unwrap();
    let root = tmp.path();

    fs::create_dir_all(root.join("src")).unwrap();
    fs::write(root.join("src/auth.ts"), "export function login() {}").unwrap();

    fs::create_dir_all(root.join("specs/auth")).unwrap();
    fs::write(
        root.join("specs/auth/auth.spec.md"),
        valid_spec("auth", &["src/auth.ts"]),
    )
    .unwrap();

    let output = specsync()
        .args(["score", "--root", root.to_str().unwrap(), "--json"])
        .output()
        .unwrap();

    let json: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    assert!(json["average_score"].is_number());
    assert!(json["grade"].is_string());
    assert!(json["specs"].is_array());
    let specs = json["specs"].as_array().unwrap();
    assert_eq!(specs.len(), 1);
    assert!(specs[0]["total"].as_u64().unwrap() > 0);
}

// ─── Diff Command Tests ─────────────────────────────────────────────────

#[test]
fn diff_shows_changes_since_base_ref() {
    let tmp = TempDir::new().unwrap();
    let root = tmp.path();

    // Initialize a git repo
    std::process::Command::new("git")
        .args(["init"])
        .current_dir(root)
        .output()
        .unwrap();
    std::process::Command::new("git")
        .args(["config", "user.email", "test@test.com"])
        .current_dir(root)
        .output()
        .unwrap();
    std::process::Command::new("git")
        .args(["config", "user.name", "Test"])
        .current_dir(root)
        .output()
        .unwrap();

    write_config(root, "specs", &["src"]);

    fs::create_dir_all(root.join("src/auth")).unwrap();
    fs::write(
        root.join("src/auth/service.ts"),
        "export function login() {}\n",
    )
    .unwrap();

    fs::create_dir_all(root.join("specs/auth")).unwrap();
    fs::write(
        root.join("specs/auth/auth.spec.md"),
        valid_spec("auth", &["src/auth/service.ts"]),
    )
    .unwrap();

    // Initial commit
    std::process::Command::new("git")
        .args(["add", "."])
        .current_dir(root)
        .output()
        .unwrap();
    std::process::Command::new("git")
        .args(["commit", "-m", "initial"])
        .current_dir(root)
        .output()
        .unwrap();

    // Add a new export after the commit
    fs::write(
        root.join("src/auth/service.ts"),
        "export function login() {}\nexport function logout() {}\n",
    )
    .unwrap();

    // Stage but don't commit — diff should detect changes
    std::process::Command::new("git")
        .args(["add", "."])
        .current_dir(root)
        .output()
        .unwrap();

    // Run diff with --json
    let output = specsync()
        .args([
            "diff",
            "--base",
            "HEAD",
            "--root",
            root.to_str().unwrap(),
            "--json",
        ])
        .output()
        .unwrap();

    assert!(output.status.success(), "diff command should succeed");
    let stdout = String::from_utf8_lossy(&output.stdout);
    let json: serde_json::Value = serde_json::from_str(stdout.trim()).unwrap();

    let changes = json["changes"].as_array().unwrap();
    assert!(!changes.is_empty(), "Expected at least one changed spec");
    assert!(
        changes[0]["new_exports"]
            .as_array()
            .unwrap()
            .iter()
            .any(|e| e.as_str() == Some("logout")),
        "Expected 'logout' in new_exports"
    );
}

#[test]
fn diff_fails_loud_on_unreadable_source_file() {
    // Regression: a changed source file whose exports can't be read (non-UTF-8)
    // silently contributed zero exports, so real new API was dropped and diff
    // reported "no drift" with exit 0. It must now surface the file and fail loud.
    let tmp = TempDir::new().unwrap();
    let root = tmp.path();
    for args in [
        vec!["init"],
        vec!["config", "user.email", "t@t.com"],
        vec!["config", "user.name", "T"],
    ] {
        std::process::Command::new("git")
            .args(&args)
            .current_dir(root)
            .output()
            .unwrap();
    }
    write_config(root, "specs", &["src"]);
    fs::create_dir_all(root.join("src")).unwrap();
    fs::write(root.join("src/api.ts"), "export function apiFn() {}\n").unwrap();
    fs::create_dir_all(root.join("specs/m")).unwrap();
    fs::write(
        root.join("specs/m/m.spec.md"),
        valid_spec("m", &["src/api.ts"]),
    )
    .unwrap();
    for args in [vec!["add", "."], vec!["commit", "-m", "init"]] {
        std::process::Command::new("git")
            .args(&args)
            .current_dir(root)
            .output()
            .unwrap();
    }

    // Rewrite api.ts with a genuinely-new export plus an invalid UTF-8 byte, then stage.
    let mut bad = b"export function apiFn() {}\nexport function brandNew() {}\n".to_vec();
    bad.push(0xFF);
    fs::write(root.join("src/api.ts"), bad).unwrap();
    std::process::Command::new("git")
        .args(["add", "."])
        .current_dir(root)
        .output()
        .unwrap();

    specsync()
        .args(["diff", "--base", "HEAD", "--root", root.to_str().unwrap()])
        .assert()
        .failure()
        .stderr(predicate::str::contains("inconclusive"));
}

#[test]
fn score_withholds_api_credit_for_unreadable_file() {
    // Regression: a `files:` entry that can't be read (here missing) produced zero
    // exports, which the API dimension scored as a PERFECT "no exports to document"
    // (20/20) — inflating the gating total. It must withhold the credit instead.
    let tmp = TempDir::new().unwrap();
    let root = tmp.path();
    write_config(root, "specs", &["src"]);
    fs::create_dir_all(root.join("src")).unwrap();
    fs::write(root.join("src/foo.rs"), "pub fn f() {}\n").unwrap();
    fs::create_dir_all(root.join("specs/foo")).unwrap();
    fs::write(
        root.join("specs/foo/foo.spec.md"),
        "---\nmodule: foo\nversion: 1\nstatus: active\nfiles:\n  - src/does_not_exist.rs\n---\n# foo\n## Purpose\np\n",
    )
    .unwrap();

    specsync()
        .args(["score", "--explain", "--root", root.to_str().unwrap()])
        .assert()
        .stdout(
            predicate::str::contains("could not analyze exports")
                .and(predicate::str::contains("no exports to document").not()),
        );
}

#[test]
fn diff_no_changes_returns_empty() {
    let tmp = TempDir::new().unwrap();
    let root = tmp.path();

    // Initialize a git repo
    std::process::Command::new("git")
        .args(["init"])
        .current_dir(root)
        .output()
        .unwrap();
    std::process::Command::new("git")
        .args(["config", "user.email", "test@test.com"])
        .current_dir(root)
        .output()
        .unwrap();
    std::process::Command::new("git")
        .args(["config", "user.name", "Test"])
        .current_dir(root)
        .output()
        .unwrap();

    write_config(root, "specs", &["src"]);

    fs::create_dir_all(root.join("src/auth")).unwrap();
    fs::write(
        root.join("src/auth/service.ts"),
        "export function login() {}\n",
    )
    .unwrap();

    fs::create_dir_all(root.join("specs/auth")).unwrap();
    fs::write(
        root.join("specs/auth/auth.spec.md"),
        valid_spec("auth", &["src/auth/service.ts"]),
    )
    .unwrap();

    // Commit everything — no changes after commit
    std::process::Command::new("git")
        .args(["add", "."])
        .current_dir(root)
        .output()
        .unwrap();
    std::process::Command::new("git")
        .args(["commit", "-m", "initial"])
        .current_dir(root)
        .output()
        .unwrap();

    // Run diff — nothing changed since HEAD
    let output = specsync()
        .args([
            "diff",
            "--base",
            "HEAD",
            "--root",
            root.to_str().unwrap(),
            "--json",
        ])
        .output()
        .unwrap();

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    let json: serde_json::Value = serde_json::from_str(stdout.trim()).unwrap();
    assert!(
        json["changes"].as_array().unwrap().is_empty(),
        "Expected no changes"
    );
}

#[test]
fn diff_bad_base_ref_fails_loud() {
    // Regression: `git diff` exits non-zero on a bad base ref with empty stdout.
    // The command must NOT report "no files changed" and exit 0 (that would silently
    // mask a failed comparison and green-light CI); it must fail loud (exit != 0).
    let tmp = TempDir::new().unwrap();
    let root = tmp.path();

    std::process::Command::new("git")
        .args(["init"])
        .current_dir(root)
        .output()
        .unwrap();
    std::process::Command::new("git")
        .args(["config", "user.email", "test@test.com"])
        .current_dir(root)
        .output()
        .unwrap();
    std::process::Command::new("git")
        .args(["config", "user.name", "Test"])
        .current_dir(root)
        .output()
        .unwrap();

    write_config(root, "specs", &["src"]);
    fs::create_dir_all(root.join("src/auth")).unwrap();
    fs::write(
        root.join("src/auth/service.ts"),
        "export function login() {}\n",
    )
    .unwrap();
    fs::create_dir_all(root.join("specs/auth")).unwrap();
    fs::write(
        root.join("specs/auth/auth.spec.md"),
        valid_spec("auth", &["src/auth/service.ts"]),
    )
    .unwrap();
    std::process::Command::new("git")
        .args(["add", "."])
        .current_dir(root)
        .output()
        .unwrap();
    std::process::Command::new("git")
        .args(["commit", "-m", "initial"])
        .current_dir(root)
        .output()
        .unwrap();

    let output = specsync()
        .args([
            "diff",
            "--base",
            "no-such-ref-xyz",
            "--root",
            root.to_str().unwrap(),
        ])
        .output()
        .unwrap();

    assert!(
        !output.status.success(),
        "diff against a bogus base ref must fail loud, not report 'no changes' and exit 0"
    );
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains("no-such-ref-xyz"),
        "error should name the bad base ref; stderr was: {stderr}"
    );
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        !stdout.contains("No files changed"),
        "must not print the no-drift message on a failed diff; stdout was: {stdout}"
    );
}

#[test]
fn diff_detects_removed_exports() {
    let tmp = TempDir::new().unwrap();
    let root = tmp.path();

    std::process::Command::new("git")
        .args(["init"])
        .current_dir(root)
        .output()
        .unwrap();
    std::process::Command::new("git")
        .args(["config", "user.email", "test@test.com"])
        .current_dir(root)
        .output()
        .unwrap();
    std::process::Command::new("git")
        .args(["config", "user.name", "Test"])
        .current_dir(root)
        .output()
        .unwrap();

    write_config(root, "specs", &["src"]);

    fs::create_dir_all(root.join("src/auth")).unwrap();
    fs::write(
        root.join("src/auth/service.ts"),
        "export function login() {}\nexport function logout() {}\n",
    )
    .unwrap();

    // Spec documents both login and logout
    fs::create_dir_all(root.join("specs/auth")).unwrap();
    let spec = r#"---
module: auth
version: 1
status: active
files:
  - src/auth/service.ts
db_tables: []
depends_on: []
---

# Auth

## Purpose

Auth module.

## Public API

| Function | Description |
|----------|-------------|
| `login` | Log in |
| `logout` | Log out |

## Invariants

1. Always valid.

## Behavioral Examples

### Scenario: Basic

- **Given** precondition
- **When** action
- **Then** result

## Error Cases

| Condition | Behavior |
|-----------|----------|

## Dependencies

None

## Change Log

| Date | Author | Change |
|------|--------|--------|
"#;
    fs::write(root.join("specs/auth/auth.spec.md"), spec).unwrap();

    // Commit with both exports
    std::process::Command::new("git")
        .args(["add", "."])
        .current_dir(root)
        .output()
        .unwrap();
    std::process::Command::new("git")
        .args(["commit", "-m", "initial"])
        .current_dir(root)
        .output()
        .unwrap();

    // Remove logout export
    fs::write(
        root.join("src/auth/service.ts"),
        "export function login() {}\n",
    )
    .unwrap();

    let output = specsync()
        .args([
            "diff",
            "--base",
            "HEAD",
            "--root",
            root.to_str().unwrap(),
            "--json",
        ])
        .output()
        .unwrap();

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    let json: serde_json::Value = serde_json::from_str(stdout.trim()).unwrap();

    let changes = json["changes"].as_array().unwrap();
    assert!(!changes.is_empty());
    assert!(
        changes[0]["removed_exports"]
            .as_array()
            .unwrap()
            .iter()
            .any(|e| e.as_str() == Some("logout")),
        "Expected 'logout' in removed_exports"
    );
}

#[test]
fn diff_human_readable_output() {
    let tmp = TempDir::new().unwrap();
    let root = tmp.path();

    std::process::Command::new("git")
        .args(["init"])
        .current_dir(root)
        .output()
        .unwrap();
    std::process::Command::new("git")
        .args(["config", "user.email", "test@test.com"])
        .current_dir(root)
        .output()
        .unwrap();
    std::process::Command::new("git")
        .args(["config", "user.name", "Test"])
        .current_dir(root)
        .output()
        .unwrap();

    write_config(root, "specs", &["src"]);

    fs::create_dir_all(root.join("src/auth")).unwrap();
    fs::write(
        root.join("src/auth/service.ts"),
        "export function login() {}\n",
    )
    .unwrap();

    fs::create_dir_all(root.join("specs/auth")).unwrap();
    fs::write(
        root.join("specs/auth/auth.spec.md"),
        valid_spec("auth", &["src/auth/service.ts"]),
    )
    .unwrap();

    std::process::Command::new("git")
        .args(["add", "."])
        .current_dir(root)
        .output()
        .unwrap();
    std::process::Command::new("git")
        .args(["commit", "-m", "initial"])
        .current_dir(root)
        .output()
        .unwrap();

    // Add new export
    fs::write(
        root.join("src/auth/service.ts"),
        "export function login() {}\nexport function signup() {}\n",
    )
    .unwrap();

    // Run without --json for human-readable output
    specsync()
        .args(["diff", "--base", "HEAD", "--root", root.to_str().unwrap()])
        .assert()
        .success()
        .stdout(predicate::str::contains("auth"))
        .stdout(predicate::str::contains("signup"));
}

#[test]
fn diff_detects_spec_file_only_changes() {
    let tmp = TempDir::new().unwrap();
    let root = tmp.path();

    std::process::Command::new("git")
        .args(["init"])
        .current_dir(root)
        .output()
        .unwrap();
    std::process::Command::new("git")
        .args(["config", "user.email", "test@test.com"])
        .current_dir(root)
        .output()
        .unwrap();
    std::process::Command::new("git")
        .args(["config", "user.name", "Test"])
        .current_dir(root)
        .output()
        .unwrap();

    write_config(root, "specs", &["src"]);

    fs::create_dir_all(root.join("src/auth")).unwrap();
    fs::write(
        root.join("src/auth/service.ts"),
        "export function login() {}\n",
    )
    .unwrap();

    fs::create_dir_all(root.join("specs/auth")).unwrap();
    fs::write(
        root.join("specs/auth/auth.spec.md"),
        valid_spec("auth", &["src/auth/service.ts"]),
    )
    .unwrap();

    std::process::Command::new("git")
        .args(["add", "."])
        .current_dir(root)
        .output()
        .unwrap();
    std::process::Command::new("git")
        .args(["commit", "-m", "initial"])
        .current_dir(root)
        .output()
        .unwrap();

    // Modify ONLY the spec file — no source file changes
    let updated_spec = valid_spec("auth", &["src/auth/service.ts"]).replace(
        "This module does something.",
        "Updated auth module description.",
    );
    fs::write(root.join("specs/auth/auth.spec.md"), &updated_spec).unwrap();

    // diff should detect the spec was modified even though no source files changed
    let output = specsync()
        .args([
            "diff",
            "--base",
            "HEAD",
            "--root",
            root.to_str().unwrap(),
            "--format",
            "json",
        ])
        .output()
        .unwrap();

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        stdout.contains("auth"),
        "Expected diff to report the auth spec when only the spec file changed. Got:\n{stdout}"
    );

    let json: serde_json::Value = serde_json::from_str(&stdout).unwrap();
    let changes = json["changes"].as_array().unwrap();
    assert_eq!(changes.len(), 1, "Expected exactly 1 change entry");
    assert_eq!(changes[0]["spec_modified"], true);
    assert!(
        changes[0]["changed_files"].as_array().unwrap().is_empty(),
        "No source files should have changed"
    );
}

// ─── specsync migrate ──────────────────────────────────────────────────

#[test]
fn migrate_full_v3_to_v4() {
    let tmp = TempDir::new().unwrap();
    let root = tmp.path().to_path_buf();
    setup_v3_project(&root);

    // Run migration
    specsync()
        .args(["migrate", "--root"])
        .arg(&root)
        .assert()
        .success()
        .stdout(predicate::str::contains("Successfully migrated to v4.0.0"));

    // Verify directory structure
    assert!(root.join(".specsync").exists(), ".specsync/ should exist");
    assert!(
        root.join(".specsync/lifecycle").exists(),
        ".specsync/lifecycle/ should exist"
    );
    assert!(
        root.join(".specsync/changes").exists(),
        ".specsync/changes/ should exist"
    );
    assert!(
        root.join(".specsync/archive").exists(),
        ".specsync/archive/ should exist"
    );

    // Config relocated
    assert!(
        root.join(".specsync/config.toml").exists(),
        "config.toml should exist"
    );
    assert!(
        !root.join("specsync.json").exists(),
        "specsync.json should be removed"
    );

    // Registry relocated
    assert!(
        root.join(".specsync/registry.toml").exists(),
        "registry.toml should exist"
    );
    assert!(
        !root.join("specsync-registry.toml").exists(),
        "specsync-registry.toml should be removed"
    );

    // Lifecycle extracted
    assert!(
        root.join(".specsync/lifecycle/auth.json").exists(),
        "lifecycle/auth.json should exist"
    );

    // Lifecycle log removed from spec frontmatter
    let spec_content = fs::read_to_string(root.join("specs/auth/auth.spec.md")).unwrap();
    assert!(
        !spec_content.contains("lifecycle_log:"),
        "lifecycle_log should be removed from spec"
    );

    // Version stamped
    let version = fs::read_to_string(root.join(".specsync/version")).unwrap();
    assert_eq!(version.trim(), "4.0.0");

    // Backup created
    assert!(
        root.join(".specsync/backup-3x/manifest.json").exists(),
        "backup manifest should exist"
    );
    assert!(
        root.join(".specsync/backup-3x/specsync.json").exists(),
        "backup of specsync.json should exist"
    );

    // Gitignore created
    assert!(
        root.join(".specsync/.gitignore").exists(),
        ".gitignore should exist"
    );
    let gitignore = fs::read_to_string(root.join(".specsync/.gitignore")).unwrap();
    assert!(
        gitignore.contains("backup-3x/"),
        "gitignore should ignore backup-3x"
    );
    // archive/ should not be gitignored (part of the v4 lifecycle)
    let archive_is_ignored = gitignore
        .lines()
        .any(|line| !line.starts_with('#') && line.trim() == "archive/");
    assert!(!archive_is_ignored, "gitignore should NOT ignore archive");
    // hashes.json SHOULD be gitignored (local-only cache, regenerated on each run)
    let hashes_is_ignored = gitignore
        .lines()
        .any(|line| !line.starts_with('#') && line.trim() == "hashes.json");
    assert!(hashes_is_ignored, "gitignore SHOULD ignore hashes.json");
    // Also check root .gitignore has .specsync/hashes.json
    let root_gitignore = fs::read_to_string(root.join(".gitignore")).unwrap_or_default();
    assert!(
        root_gitignore.contains(".specsync/hashes.json"),
        "root .gitignore should contain .specsync/hashes.json"
    );
}

#[test]
fn migrate_check_passes_after_migration() {
    let tmp = TempDir::new().unwrap();
    let root = tmp.path().to_path_buf();
    setup_v3_project(&root);

    // Migrate
    specsync()
        .args(["migrate", "--root"])
        .arg(&root)
        .assert()
        .success();

    // Check should pass on the migrated project
    specsync()
        .args(["check", "--root"])
        .arg(&root)
        .assert()
        .success()
        .stdout(predicate::str::contains("0 failed"));
}

#[test]
fn migrate_idempotent_rerun_is_noop() {
    let tmp = TempDir::new().unwrap();
    let root = tmp.path().to_path_buf();
    setup_v3_project(&root);

    // First migration
    specsync()
        .args(["migrate", "--root"])
        .arg(&root)
        .assert()
        .success()
        .stdout(predicate::str::contains("Successfully migrated"));

    // Second migration should be a no-op
    specsync()
        .args(["migrate", "--root"])
        .arg(&root)
        .assert()
        .success()
        .stdout(predicate::str::contains("Already at v4.0.0"));
}

#[test]
fn migrate_dry_run_no_side_effects() {
    let tmp = TempDir::new().unwrap();
    let root = tmp.path().to_path_buf();
    setup_v3_project(&root);

    // Dry run
    specsync()
        .args(["migrate", "--dry-run", "--root"])
        .arg(&root)
        .assert()
        .success()
        .stdout(predicate::str::contains("Dry run complete"));

    // Nothing should have changed
    assert!(
        root.join("specsync.json").exists(),
        "specsync.json should still exist after dry-run"
    );
    assert!(
        root.join("specsync-registry.toml").exists(),
        "registry should still exist after dry-run"
    );
    assert!(
        !root.join(".specsync/config.toml").exists(),
        "config.toml should NOT exist after dry-run"
    );
    assert!(
        !root.join(".specsync/version").exists(),
        "version file should NOT exist after dry-run"
    );

    // Spec should still have lifecycle_log
    let spec_content = fs::read_to_string(root.join("specs/auth/auth.spec.md")).unwrap();
    assert!(
        spec_content.contains("lifecycle_log:"),
        "lifecycle_log should still be in spec after dry-run"
    );
}

#[test]
fn migrate_json_output_format() {
    let tmp = TempDir::new().unwrap();
    let root = tmp.path().to_path_buf();
    setup_v3_project(&root);

    let output = specsync()
        .args(["migrate", "--format", "json", "--root"])
        .arg(&root)
        .output()
        .unwrap();

    assert!(output.status.success());

    let stdout = String::from_utf8(output.stdout).unwrap();
    let json: serde_json::Value = serde_json::from_str(&stdout).unwrap();
    assert_eq!(json["status"], "completed");
    assert_eq!(json["version"], "4.0.0");
    assert_eq!(json["dry_run"], false);
}

#[test]
fn migrate_no_project_fails() {
    let tmp = TempDir::new().unwrap();
    let root = tmp.path().to_path_buf();

    // Empty directory — no spec-sync project
    specsync()
        .args(["migrate", "--root"])
        .arg(&root)
        .assert()
        .failure();
}

#[test]
fn migrate_preserves_unparseable_config_and_fails() {
    // Regression: a single parse error (here a trailing comma) must not cause
    // migrate to write a pure-default config.toml, delete the original, and
    // report success. It must fail loudly and leave the project untouched.
    let tmp = TempDir::new().unwrap();
    let root = tmp.path().to_path_buf();
    let original = "{\n  \"sourceDirs\": [\"lib\"],\n  \"enforcement\": \"strict\",\n}\n";
    fs::write(root.join("specsync.json"), original).unwrap();
    fs::create_dir_all(root.join("specs")).unwrap();

    specsync()
        .args(["migrate", "--no-backup", "--root"])
        .arg(&root)
        .assert()
        .failure();

    // Original config preserved byte-for-byte; no default config written.
    assert_eq!(
        fs::read_to_string(root.join("specsync.json")).unwrap(),
        original,
        "the original (malformed) config must be left untouched"
    );
    assert!(
        !root.join(".specsync/config.toml").exists(),
        "no default config.toml should have been written"
    );
    // And no version stamp that would make a re-run refuse to migrate.
    assert!(
        !root.join(".specsync/version").exists(),
        "migration must not stamp a version when it aborted"
    );
}

#[test]
fn migrate_no_backup_flag() {
    let tmp = TempDir::new().unwrap();
    let root = tmp.path().to_path_buf();
    setup_v3_project(&root);

    specsync()
        .args(["migrate", "--no-backup", "--root"])
        .arg(&root)
        .assert()
        .success()
        .stdout(predicate::str::contains("Successfully migrated"));

    // Backup should NOT exist
    assert!(
        !root.join(".specsync/backup-3x/manifest.json").exists(),
        "backup should not exist with --no-backup"
    );

    // But migration should still be complete
    let version = fs::read_to_string(root.join(".specsync/version")).unwrap();
    assert_eq!(version.trim(), "4.0.0");
}

#[test]
fn migrate_partial_recovery() {
    let tmp = TempDir::new().unwrap();
    let root = tmp.path().to_path_buf();
    setup_v3_project(&root);

    // Simulate a partial migration: create .specsync/ with version but leave old config
    fs::create_dir_all(root.join(".specsync/lifecycle")).unwrap();
    fs::create_dir_all(root.join(".specsync/changes")).unwrap();
    fs::create_dir_all(root.join(".specsync/archive")).unwrap();
    // Don't write version file — so migrate should detect partial state and continue

    // Run migrate — should complete the remaining steps
    specsync()
        .args(["migrate", "--root"])
        .arg(&root)
        .assert()
        .success()
        .stdout(predicate::str::contains("Successfully migrated"));

    // Verify full migration completed
    assert!(root.join(".specsync/config.toml").exists());
    assert!(root.join(".specsync/version").exists());
    let version = fs::read_to_string(root.join(".specsync/version")).unwrap();
    assert_eq!(version.trim(), "4.0.0");
}

// ─── Companion file integration tests ───────────────────────────────────

#[test]
fn generate_creates_companion_files() {
    let tmp = TempDir::new().unwrap();
    let root = setup_v4_unspecced(&tmp, "");

    specsync()
        .args(["generate", "--root"])
        .arg(&root)
        .assert()
        .success()
        .stdout(predicate::str::contains("Generated"));

    let spec_dir = root.join("specs/billing");
    assert!(
        spec_dir.join("billing.spec.md").exists(),
        "spec should exist"
    );
    assert!(spec_dir.join("tasks.md").exists(), "tasks.md should exist");
    assert!(
        spec_dir.join("context.md").exists(),
        "context.md should exist"
    );
    assert!(
        spec_dir.join("requirements.md").exists(),
        "requirements.md should exist"
    );
    assert!(
        spec_dir.join("testing.md").exists(),
        "testing.md should exist"
    );
    // design.md should NOT be created by default
    assert!(
        !spec_dir.join("design.md").exists(),
        "design.md should NOT exist by default"
    );
}

#[test]
fn generate_creates_design_md_when_enabled() {
    let tmp = TempDir::new().unwrap();
    let root = setup_v4_unspecced(&tmp, "\n[companions]\ndesign = true\n");

    specsync()
        .args(["generate", "--root"])
        .arg(&root)
        .assert()
        .success()
        .stdout(predicate::str::contains("Generated"));

    let spec_dir = root.join("specs/billing");
    assert!(
        spec_dir.join("billing.spec.md").exists(),
        "spec should exist"
    );
    assert!(
        spec_dir.join("testing.md").exists(),
        "testing.md should exist"
    );
    assert!(
        spec_dir.join("design.md").exists(),
        "design.md should exist when companions.design = true"
    );

    // Verify design.md has correct frontmatter
    let design_content = fs::read_to_string(spec_dir.join("design.md")).unwrap();
    assert!(
        design_content.contains("spec: billing.spec.md"),
        "design.md should reference spec"
    );
}

#[test]
fn companion_testing_md_has_correct_structure() {
    let tmp = TempDir::new().unwrap();
    let root = setup_v4_unspecced(&tmp, "");

    specsync()
        .args(["generate", "--root"])
        .arg(&root)
        .assert()
        .success();

    let testing_content = fs::read_to_string(root.join("specs/billing/testing.md")).unwrap();
    assert!(
        testing_content.contains("spec: billing.spec.md"),
        "testing.md should reference spec"
    );
    assert!(
        testing_content.contains("## Automated Testing") || testing_content.contains("## Test"),
        "testing.md should have test-related sections"
    );
}

#[test]
fn companion_files_not_overwritten_on_regenerate() {
    let tmp = TempDir::new().unwrap();
    let root = setup_v4_unspecced(&tmp, "");

    // First generate
    specsync()
        .args(["generate", "--root"])
        .arg(&root)
        .assert()
        .success();

    // Modify a companion
    let tasks_path = root.join("specs/billing/tasks.md");
    fs::write(
        &tasks_path,
        "---\nspec: billing.spec.md\n---\n\n## Custom Content\n",
    )
    .unwrap();

    // Add a new unspecced module to trigger another generate
    fs::create_dir_all(root.join("src/shipping")).unwrap();
    fs::write(
        root.join("src/shipping/index.ts"),
        "export function ship() {}\n",
    )
    .unwrap();

    specsync()
        .args(["generate", "--root"])
        .arg(&root)
        .assert()
        .success();

    // Original companion should be untouched
    let tasks_content = fs::read_to_string(&tasks_path).unwrap();
    assert!(
        tasks_content.contains("## Custom Content"),
        "existing companion files should not be overwritten"
    );
}

// ─── specsync stale ──────────────────────────────────────────────────────

#[test]
fn stale_outside_git_repo_fails_with_message() {
    let tmp = TempDir::new().unwrap();
    let root = setup_minimal_project(&tmp);
    // No `git init` — staleness detection requires git history.

    specsync()
        .arg("stale")
        .arg("--root")
        .arg(&root)
        .assert()
        .failure()
        .stderr(predicate::str::contains("Not a git repository"));
}

#[test]
fn stale_outside_git_repo_json_reports_error() {
    let tmp = TempDir::new().unwrap();
    let root = setup_minimal_project(&tmp);

    specsync()
        .arg("stale")
        .arg("--root")
        .arg(&root)
        .arg("--format")
        .arg("json")
        .assert()
        .failure()
        .stdout(predicate::str::contains("not a git repository"))
        .stdout(predicate::str::contains("\"stale_specs\""));
}

#[test]
fn stale_in_fresh_repo_reports_all_up_to_date() {
    let tmp = TempDir::new().unwrap();
    let root = setup_minimal_project(&tmp);

    // Initialize a repo and commit everything so source and spec share history.
    let git = |args: &[&str]| {
        Command::new("git")
            .args(args)
            .current_dir(&root)
            .assert()
            .success();
    };
    git(&["init"]);
    git(&["config", "user.email", "test@test.com"]);
    git(&["config", "user.name", "Test"]);
    git(&["config", "commit.gpgsign", "false"]);
    git(&["add", "-A"]);
    git(&["commit", "-m", "initial"]);

    specsync()
        .arg("stale")
        .arg("--root")
        .arg(&root)
        .assert()
        .success()
        .stdout(predicate::str::contains("up to date"));
}

// ─── specsync merge ─────────────────────────────────────────────────────

/// Regression (CRITICAL): `merge` must never write a corrupt spec. A conflict
/// hunk that swallows the `---` fences previously resolved to loose/doubled/empty
/// frontmatter written as "✓ resolved". We now leave such hunks for manual
/// resolution — the invariant: a marker-free result is always valid frontmatter,
/// and the body is never deleted.
#[test]
fn merge_never_writes_corrupt_spec_for_fence_hunk() {
    let tmp = TempDir::new().unwrap();
    let root = tmp.path();
    write_config(root, "specs", &["src"]);
    fs::create_dir_all(root.join("src")).unwrap();
    fs::write(
        root.join("src/minimal.ts"),
        "export function doThing() {}\n",
    )
    .unwrap();
    fs::create_dir_all(root.join("specs/minimal")).unwrap();

    // Each side of the conflict carries its own `---` frontmatter fences.
    let conflicted = "\
<<<<<<< HEAD
---
module: minimal
version: 2
status: active
files:
  - src/minimal.ts
db_tables: []
depends_on: []
---
=======
---
module: minimal
version: 1
status: active
files:
  - src/minimal.ts
db_tables: []
depends_on: []
---
>>>>>>> branch
# Minimal

## Purpose

Minimal module.
";
    let spec_path = root.join("specs/minimal/minimal.spec.md");
    fs::write(&spec_path, conflicted).unwrap();

    // May resolve or defer to manual — but must never corrupt or delete the body.
    let _ = specsync()
        .current_dir(root)
        .args(["merge", "--all"])
        .assert();

    let after = fs::read_to_string(&spec_path).unwrap();
    if !after.contains("<<<<<<<") {
        assert!(
            after.starts_with("---\n") && after.contains("module: minimal"),
            "merge produced a corrupt, marker-free spec:\n{after}"
        );
    }
    assert!(
        after.contains("## Purpose") && after.contains("Minimal module."),
        "spec body must never be deleted:\n{after}"
    );
}

/// The common case must still auto-resolve: two branches bumped `version`, with
/// the `---` fences left in the surrounding clean regions.
#[test]
fn merge_resolves_interior_field_conflict() {
    let tmp = TempDir::new().unwrap();
    let root = tmp.path();
    write_config(root, "specs", &["src"]);
    fs::create_dir_all(root.join("src")).unwrap();
    fs::write(
        root.join("src/minimal.ts"),
        "export function doThing() {}\n",
    )
    .unwrap();
    fs::create_dir_all(root.join("specs/minimal")).unwrap();

    // The conflict is purely the `version` line; fences stay in clean regions.
    let conflicted = "\
---
module: minimal
<<<<<<< HEAD
version: 2
=======
version: 3
>>>>>>> branch
status: active
files:
  - src/minimal.ts
db_tables: []
depends_on: []
---
# Minimal

## Purpose

Minimal module.
";
    let spec_path = root.join("specs/minimal/minimal.spec.md");
    fs::write(&spec_path, conflicted).unwrap();

    specsync()
        .current_dir(root)
        .args(["merge", "--all"])
        .assert()
        .success()
        .stdout(predicate::str::contains("Auto-resolved"))
        .stdout(predicate::str::contains("Auto-resolvable").not());

    let resolved = fs::read_to_string(&spec_path).unwrap();
    assert!(
        !resolved.contains("<<<<<<<"),
        "interior field conflict should auto-resolve, got:\n{resolved}"
    );
    assert!(
        resolved.starts_with("---\n") && resolved.contains("version: 3"),
        "resolved spec must be valid frontmatter with theirs' version, got:\n{resolved}"
    );
    // No "Frontmatter invalid" — the resolved spec parses.
    specsync()
        .current_dir(root)
        .arg("check")
        .assert()
        .stdout(predicate::str::contains("Frontmatter invalid").not());
}

#[test]
fn merge_issue_427_diff3_max_version_and_dry_run_are_lossless() {
    let tmp = TempDir::new().unwrap();
    let root = tmp.path();
    write_config(root, "specs", &["src"]);
    fs::create_dir_all(root.join("src")).unwrap();
    fs::write(
        root.join("src/minimal.ts"),
        "export function doThing() {}\n",
    )
    .unwrap();
    fs::create_dir_all(root.join("specs/minimal")).unwrap();

    let conflicted = concat!(
        "---\nmodule: minimal\n",
        "<<<<",
        "<<< HEAD\nversion: 3\n",
        "||||",
        "||| base\nversion: 1\n",
        "===",
        "====\nversion: 2\n",
        ">>>>",
        ">>> branch\n",
        "status: active\nfiles:\n  - src/minimal.ts\ndb_tables: []\ndepends_on: []\n",
        "---\n# Minimal\n\n## Purpose\n\nMinimal module.\n",
    );
    let spec_path = root.join("specs/minimal/minimal.spec.md");
    fs::write(&spec_path, conflicted).unwrap();

    specsync()
        .current_dir(root)
        .args(["merge", "--all", "--dry-run"])
        .assert()
        .success()
        .stdout(predicate::str::contains("would resolve"))
        .stdout(predicate::str::contains("Auto-resolvable"));
    assert_eq!(fs::read_to_string(&spec_path).unwrap(), conflicted);

    specsync()
        .current_dir(root)
        .args(["merge", "--all"])
        .assert()
        .success();
    let resolved = fs::read_to_string(&spec_path).unwrap();
    assert!(resolved.contains("version: 3"), "{resolved}");
    assert!(!resolved.contains("version: 2"), "{resolved}");
    assert!(!resolved.contains("|||||||"), "{resolved}");
    assert!(!resolved.contains("<<<<<<<"), "{resolved}");
}

#[test]
fn merge_issue_427_mixed_manual_file_remains_byte_identical() {
    let tmp = TempDir::new().unwrap();
    let root = tmp.path();
    write_config(root, "specs", &["src"]);
    fs::create_dir_all(root.join("src")).unwrap();
    fs::write(
        root.join("src/minimal.ts"),
        "export function doThing() {}\n",
    )
    .unwrap();
    fs::create_dir_all(root.join("specs/minimal")).unwrap();

    let conflicted = concat!(
        "---\nmodule: minimal\n",
        "<<<<",
        "<<< HEAD\nversion: 3\n",
        "===",
        "====\nversion: 2\n",
        ">>>>",
        ">>> branch\n",
        "status: active\nfiles:\n  - src/minimal.ts\ndb_tables: []\ndepends_on: []\n",
        "---\n# Minimal\n\n## Public API\n\n| Name | Description |\n|------|-------------|\n",
        "<<<<",
        "<<< HEAD\n| `doThing` | Main description. |\n",
        "===",
        "====\n| `doThing` | Incoming description. |\n",
        ">>>>",
        ">>> branch\n",
    );
    let spec_path = root.join("specs/minimal/minimal.spec.md");
    fs::write(&spec_path, conflicted).unwrap();

    specsync()
        .current_dir(root)
        .args(["merge", "--all"])
        .assert()
        .failure()
        .stdout(predicate::str::contains("Auto-resolvable"))
        .stdout(predicate::str::contains("HEAD"))
        .stdout(predicate::str::contains("branch"))
        .stdout(predicate::str::contains("left unchanged (all-or-nothing)"));

    assert_eq!(fs::read_to_string(&spec_path).unwrap(), conflicted);
}

// ─── specsync hooks ─────────────────────────────────────────────────────

#[test]
fn hooks_uninstall_preserves_user_content_after_block() {
    // Regression: `hooks uninstall` used to delete from the managed block to EOF,
    // wiping any content the user added after it (and the whole file if spec-sync
    // created it). It must now remove only the managed block.
    let tmp = TempDir::new().unwrap();
    let root = tmp.path();

    specsync()
        .current_dir(root)
        .args(["hooks", "install", "--claude"])
        .assert()
        .success();

    let claude = root.join("CLAUDE.md");
    let mut content = fs::read_to_string(&claude).unwrap();
    content.push_str("\n## Deploy Notes\nDO NOT DELETE THIS LINE\n");
    fs::write(&claude, content).unwrap();

    specsync()
        .current_dir(root)
        .args(["hooks", "uninstall", "--claude"])
        .assert()
        .success();

    assert!(claude.exists(), "CLAUDE.md must not be deleted");
    let after = fs::read_to_string(&claude).unwrap();
    assert!(
        after.contains("DO NOT DELETE THIS LINE"),
        "content added after the managed block must survive uninstall:\n{after}"
    );
    assert!(
        !after.contains("Spec-Sync"),
        "the managed block must be removed:\n{after}"
    );
}

// ─── specsync score: gate flags are honored, not silent no-ops ───────────

#[test]
fn score_honors_require_coverage_gate() {
    // Regression (H5): the global --require-coverage / --enforcement flags were
    // silently ignored by `score` (it always exited 0). They must now gate.
    let tmp = TempDir::new().unwrap();
    let root = tmp.path();
    write_config(root, "specs", &["src"]);
    fs::create_dir_all(root.join("src")).unwrap();
    // 1 specced + 1 unspecced file → below 100% coverage.
    fs::write(root.join("src/a.ts"), "export function a() {}\n").unwrap();
    fs::write(root.join("src/uncovered.ts"), "export function b() {}\n").unwrap();
    fs::create_dir_all(root.join("specs/a")).unwrap();
    fs::write(
        root.join("specs/a/a.spec.md"),
        valid_spec("a", &["src/a.ts"]),
    )
    .unwrap();

    // Gate flags now fail; default score stays advisory (exit 0).
    specsync()
        .current_dir(root)
        .args(["score", "--require-coverage", "100"])
        .assert()
        .failure();
    specsync()
        .current_dir(root)
        .args(["score", "--enforcement", "enforce-new"])
        .assert()
        .failure();
    // JSON output must still gate AND remain valid JSON.
    specsync()
        .current_dir(root)
        .args(["score", "--require-coverage", "100", "--format", "json"])
        .assert()
        .failure()
        .stdout(predicate::str::starts_with("{"));
    // CSV is a machine format too: it must gate WITHOUT the human failure message
    // leaking into the CSV body (regression guard for the review's CSV nit).
    specsync()
        .current_dir(root)
        .args(["score", "--require-coverage", "100", "--format", "csv"])
        .assert()
        .failure()
        .stdout(predicate::str::contains("--require-coverage").not());
    specsync().current_dir(root).arg("score").assert().success();
}

#[test]
fn score_no_specs_still_evaluates_requested_gate() {
    // Regression (H5/H2 class): a spec-less project must still FAIL a requested
    // gate rather than taking the no-spec early-exit, while a plain `score`
    // keeps its friendly early-exit.
    let tmp = TempDir::new().unwrap();
    let root = tmp.path();
    write_config(root, "specs", &["src"]);
    fs::create_dir_all(root.join("src")).unwrap();
    fs::write(root.join("src/a.ts"), "export function a() {}\n").unwrap();

    specsync()
        .current_dir(root)
        .args(["score", "--require-coverage", "100"])
        .assert()
        .failure();
    specsync().current_dir(root).arg("score").assert().success();
}

#[test]
fn check_scalar_inline_comment_does_not_hide_specs() {
    // Regression (#6): an inline comment on `specs_dir` used to be kept in the
    // value (`"specs" # note`), mis-resolving the specs dir so every spec became
    // invisible and `check` silently passed. The spec must be discovered.
    let tmp = TempDir::new().unwrap();
    let root = tmp.path();
    fs::create_dir_all(root.join("mydocs")).unwrap();
    fs::create_dir_all(root.join("src")).unwrap();
    fs::write(root.join("src/a.ts"), "export function a() {}\n").unwrap();
    fs::write(
        root.join(".specsync.toml"),
        "specs_dir = \"mydocs\" # where specs live\nsource_dirs = [\"src\"]\n",
    )
    .unwrap();
    fs::write(
        root.join("mydocs/a.spec.md"),
        "---\nmodule: a\nstatus: stable\nfiles:\n  - src/a.ts\n---\n# A\n## Purpose\nx\n",
    )
    .unwrap();

    // The spec is now discovered (output names it) rather than "No spec files found".
    specsync()
        .current_dir(root)
        .arg("check")
        .assert()
        .stdout(predicate::str::contains("a.spec.md"));
}

#[test]
fn coverage_no_specs_evaluates_gate() {
    // Regression (M1): `coverage` used to take the no-spec early-exit (exit 0) and
    // its JSON path always exited 0, so the gate was never evaluated. A project
    // with source but no specs is 0% covered and must FAIL a requested gate.
    let tmp = TempDir::new().unwrap();
    let root = tmp.path();
    write_config(root, "specs", &["src"]);
    fs::create_dir_all(root.join("src")).unwrap();
    fs::write(root.join("src/a.ts"), "export function a() {}\n").unwrap();

    specsync()
        .current_dir(root)
        .args(["coverage", "--require-coverage", "100"])
        .assert()
        .failure();
    // JSON path must gate too, and stay valid JSON.
    specsync()
        .current_dir(root)
        .args(["coverage", "--require-coverage", "100", "--format", "json"])
        .assert()
        .failure()
        .stdout(predicate::str::starts_with("{"));
    specsync()
        .current_dir(root)
        .args(["coverage", "--enforcement", "enforce-new"])
        .assert()
        .failure();
    // A CONFIG-only enforce-new gate (no CLI flag) must also fire.
    fs::write(
        root.join(".specsync.toml"),
        "enforcement = \"enforce-new\"\nspecs_dir = \"specs\"\nsource_dirs = [\"src\"]\n",
    )
    .unwrap();
    specsync()
        .current_dir(root)
        .arg("coverage")
        .assert()
        .failure();
    // Back to a warn config → coverage report still exits 0 (no gate requested).
    fs::write(
        root.join(".specsync.toml"),
        "enforcement = \"warn\"\nspecs_dir = \"specs\"\nsource_dirs = [\"src\"]\n",
    )
    .unwrap();
    specsync()
        .current_dir(root)
        .arg("coverage")
        .assert()
        .success();
}

#[test]
fn score_config_only_enforcement_gates_no_specs() {
    // Regression (M1 sibling): a CONFIG-level enforcement gate (no CLI flag) must
    // also stop the no-spec early-exit — `score` on a spec-less project whose
    // config sets enforce-new must FAIL, matching `check`.
    let tmp = TempDir::new().unwrap();
    let root = tmp.path();
    fs::create_dir_all(root.join("src")).unwrap();
    fs::write(root.join("src/a.ts"), "export function a() {}\n").unwrap();

    fs::write(
        root.join(".specsync.toml"),
        "enforcement = \"enforce-new\"\nspecs_dir = \"specs\"\nsource_dirs = [\"src\"]\n",
    )
    .unwrap();
    specsync().current_dir(root).arg("score").assert().failure();

    // A warn config keeps the friendly advisory early-exit (exit 0).
    fs::write(
        root.join(".specsync.toml"),
        "enforcement = \"warn\"\nspecs_dir = \"specs\"\nsource_dirs = [\"src\"]\n",
    )
    .unwrap();
    specsync().current_dir(root).arg("score").assert().success();
}

// ─── specsync deps: --strict gates on undeclared-import warnings ─────────

#[test]
fn deps_strict_gates_on_undeclared_imports() {
    // Regression (H6): `deps --strict` was a silent no-op — undeclared imports
    // were reported as warnings but never failed the exit code.
    let tmp = TempDir::new().unwrap();
    let root = tmp.path();
    setup_undeclared_import_project(root);

    // Default deps is advisory (reports the warning, exits 0).
    specsync().current_dir(root).arg("deps").assert().success();
    // --strict fails on the undeclared import; non-JSON formats get the human
    // "treated as errors" note on stderr.
    specsync()
        .current_dir(root)
        .args(["deps", "--strict"])
        .assert()
        .failure()
        .stderr(predicate::str::contains("treated as errors"));
    // JSON output gates AND stays fully machine-readable: stdout is parseable
    // JSON carrying the warning, and the human strict note is suppressed
    // entirely — not even on stderr — so a JSON consumer sees only structured
    // data plus the exit code (no ANSI, nothing to parse around).
    let output = specsync()
        .current_dir(root)
        .args(["deps", "--strict", "--format", "json"])
        .assert()
        .failure()
        .get_output()
        .clone();
    let parsed: serde_json::Value = serde_json::from_slice(&output.stdout)
        .expect("deps --strict json stdout must be valid JSON");
    assert!(
        !parsed["undeclared_imports"].as_array().unwrap().is_empty(),
        "expected the undeclared import to be reported in JSON"
    );
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        !stderr.contains("treated as errors") && !stderr.contains("--strict mode"),
        "JSON mode must not emit the human strict note, even on stderr; got: {stderr:?}"
    );
}

#[test]
fn deps_strict_passes_when_dependency_is_declared() {
    // No false failure: once `api` declares `depends_on: [db]`, --strict is clean.
    let tmp = TempDir::new().unwrap();
    let root = tmp.path();
    setup_undeclared_import_project(root);
    fs::write(
        root.join("specs/api/api.spec.md"),
        "---\nmodule: api\nversion: 1\nstatus: active\nfiles:\n  - src/api/api.ts\ndepends_on:\n  - db\n---\n# api\n## Purpose\np\n",
    )
    .unwrap();

    specsync()
        .current_dir(root)
        .args(["deps", "--strict"])
        .assert()
        .success();
}

#[test]
fn deps_strict_gates_on_undeclared_kotlin_import() {
    // Regression (#477): `deps` had no Kotlin import extractor at all, so a
    // Kotlin tree produced zero edges and `--strict` called the unexamined
    // graph valid — the same shape that already failed in Python.
    let tmp = TempDir::new().unwrap();
    let root = tmp.path();
    write_config(root, "specs", &["src"]);
    fs::create_dir_all(root.join("src/main/kotlin/com/example/core")).unwrap();
    fs::create_dir_all(root.join("src/main/kotlin/com/example/feature")).unwrap();
    fs::write(
        root.join("src/main/kotlin/com/example/core/Core.kt"),
        "package com.example.core\n\nclass Core\n",
    )
    .unwrap();
    fs::write(
        root.join("src/main/kotlin/com/example/feature/Feature.kt"),
        "package com.example.feature\n\nimport com.example.core.Core\n\nclass Feature(val core: Core)\n",
    )
    .unwrap();
    fs::create_dir_all(root.join("specs/core")).unwrap();
    fs::create_dir_all(root.join("specs/feature")).unwrap();
    fs::write(
        root.join("specs/core/core.spec.md"),
        valid_spec("core", &["src/main/kotlin/com/example/core/Core.kt"]),
    )
    .unwrap();
    fs::write(
        root.join("specs/feature/feature.spec.md"),
        valid_spec(
            "feature",
            &["src/main/kotlin/com/example/feature/Feature.kt"],
        ),
    )
    .unwrap();

    specsync()
        .current_dir(root)
        .args(["deps", "--strict"])
        .assert()
        .failure()
        .stdout(predicate::str::contains(
            "source imports 'core' but it is not in depends_on",
        ));

    // Declaring the dependency clears it — the analysis honours depends_on.
    fs::write(
        root.join("specs/feature/feature.spec.md"),
        valid_spec(
            "feature",
            &["src/main/kotlin/com/example/feature/Feature.kt"],
        )
        .replace("depends_on: []", "depends_on:\n  - core"),
    )
    .unwrap();
    specsync()
        .current_dir(root)
        .args(["deps", "--strict"])
        .assert()
        .success();
}

#[test]
fn deps_discloses_languages_it_cannot_analyse() {
    // A language with no import extractor contributes no edges. `deps` must say
    // so rather than let the silence read as "no undeclared imports" (#477).
    let tmp = TempDir::new().unwrap();
    let root = tmp.path();
    write_config(root, "specs", &["src"]);
    fs::create_dir_all(root.join("src")).unwrap();
    fs::write(root.join("src/core.go"), "package core\n").unwrap();
    fs::create_dir_all(root.join("specs/core")).unwrap();
    fs::write(
        root.join("specs/core/core.spec.md"),
        valid_spec("core", &["src/core.go"]),
    )
    .unwrap();

    // Disclosed, but advisory: an unanalysable language is not a failure.
    specsync()
        .current_dir(root)
        .args(["deps", "--strict"])
        .assert()
        .success()
        .stdout(
            predicate::str::contains("Import analysis is not implemented for Go (1 file(s))").and(
                predicate::str::contains(
                    "All dependency declarations are valid for the languages analysed.",
                ),
            ),
        );

    let output = specsync()
        .current_dir(root)
        .args(["deps", "--format", "json"])
        .assert()
        .success()
        .get_output()
        .clone();
    let parsed: serde_json::Value =
        serde_json::from_slice(&output.stdout).expect("deps json stdout must be valid JSON");
    assert_eq!(
        parsed["unanalyzed_languages"],
        serde_json::json!([{"language": "Go", "files": 1}]),
        "JSON consumers must be able to tell 'not analysed' from 'clean'"
    );
}

#[test]
fn deps_does_not_disclose_languages_that_have_no_imports() {
    // The disclosure must name languages whose imports went unread — not every
    // file whose extension happens to map to a Language. A YAML file has no
    // imports to miss and a shell script names a path, so a project like this
    // one is fully analysed and must say so without hedging (#477).
    let tmp = TempDir::new().unwrap();
    let root = tmp.path();
    write_config(root, "specs", &["src"]);
    fs::create_dir_all(root.join("src")).unwrap();
    fs::write(root.join("src/core.rs"), "pub fn run() {}\n").unwrap();
    fs::write(root.join("src/ci.yml"), "jobs: {}\n").unwrap();
    fs::write(root.join("src/tool.sh"), "#!/usr/bin/env bash\ntrue\n").unwrap();
    fs::create_dir_all(root.join("specs/core")).unwrap();
    fs::write(
        root.join("specs/core/core.spec.md"),
        valid_spec("core", &["src/core.rs", "src/ci.yml", "src/tool.sh"]),
    )
    .unwrap();

    specsync()
        .current_dir(root)
        .args(["deps", "--strict"])
        .assert()
        .success()
        .stdout(
            predicate::str::contains("All dependency declarations are valid.")
                .and(predicate::str::contains("Import analysis is not implemented").not()),
        );
}

#[test]
fn deps_reports_kotlin_imports_it_could_not_resolve() {
    // The fix for #477 collected Kotlin imports and then dropped every one it
    // could not map to a module, reporting the remainder as a clean graph. An
    // import the tool could not resolve must be distinguishable in the output
    // from an import that resolves to nothing.
    let tmp = TempDir::new().unwrap();
    let root = tmp.path();
    write_config(root, "specs", &["src"]);
    fs::create_dir_all(root.join("src/kt")).unwrap();
    fs::write(
        root.join("src/kt/Feature.kt"),
        "package com.example.feature\n\nimport com.example.internal.Detail\n",
    )
    .unwrap();
    fs::create_dir_all(root.join("specs/feature")).unwrap();
    fs::write(
        root.join("specs/feature/feature.spec.md"),
        valid_spec("feature", &["src/kt/Feature.kt"]),
    )
    .unwrap();

    // Advisory, like the unanalysed-language note: disclosed, not gated.
    specsync()
        .current_dir(root)
        .args(["deps", "--strict"])
        .assert()
        .success()
        .stdout(
            predicate::str::contains(
                "1 import(s) could not be mapped to a spec module, so they were not checked \
                 against depends_on: feature imports com.example.internal",
            )
            .and(predicate::str::contains(
                "All dependency declarations are valid for the imports that resolved.",
            )),
        );

    let output = specsync()
        .current_dir(root)
        .args(["deps", "--format", "json"])
        .assert()
        .success()
        .get_output()
        .clone();
    let parsed: serde_json::Value =
        serde_json::from_slice(&output.stdout).expect("deps json stdout must be valid JSON");
    assert_eq!(
        parsed["unresolved_imports"],
        serde_json::json!([{"module": "feature", "import": "com.example.internal"}]),
        "JSON consumers must be able to tell 'could not resolve' from 'nothing to resolve'"
    );
    assert_eq!(
        parsed["undeclared_imports"],
        serde_json::json!([]),
        "an unresolved import is not evidence of an undeclared dependency"
    );
}

#[test]
fn deps_fails_loud_on_unreadable_source_file() {
    // Regression: a declared source file that can't be read as UTF-8 silently
    // contributed no imports, so `deps` could pass while hiding undeclared imports.
    let tmp = TempDir::new().unwrap();
    let root = tmp.path();
    write_config(root, "specs", &["src"]);
    fs::create_dir_all(root.join("src")).unwrap();
    let mut bad = b"export function apiFn() {}\n".to_vec();
    bad.push(0xFF);
    fs::write(root.join("src/a.ts"), bad).unwrap();
    fs::create_dir_all(root.join("specs/m")).unwrap();
    fs::write(
        root.join("specs/m/m.spec.md"),
        valid_spec("m", &["src/a.ts"]),
    )
    .unwrap();

    specsync()
        .current_dir(root)
        .arg("deps")
        .assert()
        .failure()
        .stdout(predicate::str::contains(
            "could not be read as UTF-8 for dependency analysis",
        ));
}

#[test]
fn deps_fails_loud_on_unreadable_spec_file() {
    // Regression: a spec file that can't be read as UTF-8 was silently dropped from
    // the dependency graph, defeating cycle / missing-dep detection for that module.
    let tmp = TempDir::new().unwrap();
    let root = tmp.path();
    write_config(root, "specs", &["src"]);
    fs::create_dir_all(root.join("src")).unwrap();
    fs::write(root.join("src/a.rs"), "pub fn f() {}\n").unwrap();
    fs::create_dir_all(root.join("specs/m")).unwrap();
    let mut bad = b"---\nmodule: m\nfiles:\n  - src/a.rs\n---\n# m\n".to_vec();
    bad.push(0xFF);
    fs::write(root.join("specs/m/m.spec.md"), bad).unwrap();

    specsync()
        .current_dir(root)
        .arg("deps")
        .assert()
        .failure()
        .stdout(predicate::str::contains(
            "spec file could not be read as UTF-8",
        ));
}

#[test]
fn config_warns_on_unreadable_config_file() {
    // Regression: a config file that exists but can't be read as UTF-8 silently
    // reverted to built-in defaults, downgrading enforcement with no signal.
    let tmp = TempDir::new().unwrap();
    let root = tmp.path();
    fs::create_dir_all(root.join("src")).unwrap();
    fs::write(root.join("src/a.rs"), "pub fn f() {}\n").unwrap();
    // A config whose keys are valid ASCII but whose tail is invalid UTF-8.
    let mut bad = b"specs_dir = \"specs\"\nsource_dirs = [\"src\"]\n".to_vec();
    bad.extend_from_slice(&[0xFF, 0xFE]);
    fs::write(root.join(".specsync.toml"), bad).unwrap();

    specsync()
        .current_dir(root)
        .arg("check")
        .assert()
        .stderr(predicate::str::contains("exists but could not be read"));
}

#[test]
fn deps_strict_mermaid_still_gates() {
    // Regression: `deps --mermaid`/`--dot` early-returned before the strict gate, so
    // `deps --strict --mermaid` silently exited 0 on the same undeclared import that
    // `deps --strict` fails on. The diagram must still print, but the gate must apply.
    let tmp = TempDir::new().unwrap();
    let root = tmp.path();
    setup_undeclared_import_project(root);

    // Diagram is emitted on stdout AND the strict gate fails.
    let output = specsync()
        .current_dir(root)
        .args(["deps", "--strict", "--mermaid"])
        .assert()
        .failure()
        .get_output()
        .clone();
    assert!(
        String::from_utf8_lossy(&output.stdout).contains("graph LR"),
        "the mermaid diagram must still be printed to stdout"
    );
    // Without --strict, a render is advisory (exit 0).
    specsync()
        .current_dir(root)
        .args(["deps", "--mermaid"])
        .assert()
        .success();
}

// ─── generate: --format json honors the same gates as the text path ──────

#[test]
fn generate_json_honors_require_coverage_gate() {
    // Regression: `generate --format json` did not gate on
    // --require-coverage/--enforcement/--strict — a machine-consumer
    // false pass. Here an empty source dir yields vacuous 0/0 coverage that
    // --require-coverage 50 must fail loud on, in JSON just like text.
    let tmp = TempDir::new().unwrap();
    let root = tmp.path();
    fs::create_dir_all(root.join(".specsync")).unwrap();
    fs::write(
        root.join(".specsync/config.toml"),
        "specs_dir = \"specs\"\nsource_dirs = [\"src\"]\n",
    )
    .unwrap();
    fs::create_dir_all(root.join("src")).unwrap();
    fs::create_dir_all(root.join("specs")).unwrap();

    // Text path fails the gate.
    specsync()
        .current_dir(root)
        .args(["generate", "--require-coverage", "50"])
        .assert()
        .failure();
    // JSON path fails identically AND stdout stays valid JSON.
    let output = specsync()
        .current_dir(root)
        .args(["generate", "--require-coverage", "50", "--format", "json"])
        .assert()
        .failure()
        .get_output()
        .clone();
    serde_json::from_slice::<serde_json::Value>(&output.stdout)
        .expect("generate --format json stdout must be valid JSON even when the gate fails");
}

#[test]
fn generate_json_honors_enforcement_strict() {
    // An existing spec with a real validation error (a missing source file) that
    // `generate` cannot fix must fail `--enforcement strict` on the JSON path too.
    let tmp = TempDir::new().unwrap();
    let root = tmp.path();
    fs::create_dir_all(root.join(".specsync")).unwrap();
    fs::write(
        root.join(".specsync/config.toml"),
        "specs_dir = \"specs\"\nsource_dirs = [\"src\"]\n",
    )
    .unwrap();
    fs::create_dir_all(root.join("src")).unwrap();
    fs::write(root.join("src/foo.rs"), "pub fn f() {}\n").unwrap();
    fs::create_dir_all(root.join("specs/foo")).unwrap();
    fs::write(
        root.join("specs/foo/foo.spec.md"),
        "---\nmodule: foo\nversion: 1\nstatus: active\nfiles:\n  - src/foo.rs\n  - src/does_not_exist.rs\n---\n# foo\n## Purpose\np\n",
    )
    .unwrap();

    specsync()
        .current_dir(root)
        .args(["generate", "--enforcement", "strict"])
        .assert()
        .failure();
    let output = specsync()
        .current_dir(root)
        .args(["generate", "--enforcement", "strict", "--format", "json"])
        .assert()
        .failure()
        .get_output()
        .clone();
    serde_json::from_slice::<serde_json::Value>(&output.stdout)
        .expect("generate --format json stdout must be valid JSON even when the gate fails");
}

#[test]
fn generate_json_no_specs_emits_valid_json() {
    // Regression: the "No existing specs found…" diagnostic was printed to stdout even
    // under --format json, prepending non-JSON text and breaking any parser.
    let tmp = TempDir::new().unwrap();
    let root = tmp.path();
    fs::create_dir_all(root.join(".specsync")).unwrap();
    fs::write(
        root.join(".specsync/config.toml"),
        "specs_dir = \"specs\"\nsource_dirs = [\"src\"]\n",
    )
    .unwrap();
    fs::create_dir_all(root.join("src")).unwrap();
    fs::write(root.join("src/foo.js"), "export function add() {}\n").unwrap();

    let output = specsync()
        .current_dir(root)
        .args(["generate", "--format", "json"])
        .assert()
        .get_output()
        .clone();
    serde_json::from_slice::<serde_json::Value>(&output.stdout).expect(
        "generate --format json stdout must be a clean JSON document with no specs present",
    );
}

// ─── compact: idempotent files and truthful text output ───────────────
// Requirement evidence: REQ-cli-007, REQ-archive-001,
// REQ-cmd-archive-tasks-001, REQ-cmd-compact-001, and REQ-compact-001.

fn setup_compact_project(tmp: &TempDir) -> std::path::PathBuf {
    let root = tmp.path().to_path_buf();
    write_config(&root, "specs", &["src"]);
    fs::create_dir_all(root.join("src/history")).unwrap();
    fs::write(
        root.join("src/history/mod.ts"),
        "export function record() {}\n",
    )
    .unwrap();
    fs::create_dir_all(root.join("specs/history")).unwrap();

    let mut spec = valid_spec("history", &["src/history/mod.ts"]);
    for day in 1..=4 {
        spec.push_str(&format!(
            "| 2026-07-{day:02} | maintainer | Change {day} |\n"
        ));
    }
    fs::write(root.join("specs/history/history.spec.md"), spec).unwrap();
    root
}

#[test]
fn compact_dry_run_reports_counts_without_writing() {
    let tmp = TempDir::new().unwrap();
    let root = setup_compact_project(&tmp);
    let spec_path = root.join("specs/history/history.spec.md");
    let before = fs::read(&spec_path).unwrap();

    specsync()
        .args(["compact", "--keep", "2", "--dry-run", "--root"])
        .arg(&root)
        .assert()
        .success()
        .stdout(predicate::str::contains(
            "Dry run — no files will be modified",
        ))
        .stdout(predicate::str::contains("would compact 2 entries (kept 2)"))
        .stdout(predicate::str::contains(
            "Would compact 2 entries across 1 spec",
        ));

    assert_eq!(fs::read(spec_path).unwrap(), before);
}

#[test]
fn compact_cli_is_idempotent_and_preserves_trailing_newline() {
    let tmp = TempDir::new().unwrap();
    let root = setup_compact_project(&tmp);
    let spec_path = root.join("specs/history/history.spec.md");

    specsync()
        .args(["compact", "--keep", "2", "--root"])
        .arg(&root)
        .assert()
        .success()
        .stdout(predicate::str::contains("compacted 2 entries (kept 2)"))
        .stdout(predicate::str::contains(
            "Compacted 2 entries across 1 spec",
        ));

    let once = fs::read(&spec_path).unwrap();
    assert!(
        once.ends_with(b"\n"),
        "compact stripped the trailing newline"
    );

    specsync()
        .args(["compact", "--keep", "2", "--root"])
        .arg(&root)
        .assert()
        .success()
        .stdout(predicate::str::contains(
            "No changelogs need compaction (all within limit).",
        ));

    assert_eq!(
        fs::read(spec_path).unwrap(),
        once,
        "the second compact run changed the file"
    );
}

#[test]
fn compact_json_formats_are_clean_truthful_and_equivalent() {
    let tmp = TempDir::new().unwrap();
    let root = setup_compact_project(&tmp);
    let spec_path = root.join("specs/history/history.spec.md");
    let before = fs::read(&spec_path).unwrap();
    let mut outputs = Vec::new();

    for format_args in [vec!["--format", "json"], vec!["--json"]] {
        let output = specsync()
            .args(["compact", "--keep", "2", "--dry-run"])
            .args(format_args)
            .arg("--root")
            .arg(&root)
            .assert()
            .success()
            .get_output()
            .clone();
        assert!(
            !output.stdout.contains(&0x1b),
            "JSON stdout contained an ANSI escape"
        );
        outputs.push(
            serde_json::from_slice::<serde_json::Value>(&output.stdout)
                .expect("compact JSON stdout must be one valid document"),
        );
    }

    assert_eq!(outputs[0], outputs[1], "--json must match --format json");
    let result = &outputs[0];
    assert_eq!(result["command"], "compact");
    assert_eq!(result["dry_run"], true);
    assert_eq!(result["would_change"], true);
    assert_eq!(result["applied"], false);
    assert_eq!(result["complete"], true);
    assert_eq!(result["partial"], false);
    assert_eq!(result["operations"]["planned"], 1);
    assert_eq!(result["operations"]["succeeded"], 0);
    assert_eq!(result["operations"]["failed"], 0);
    assert_eq!(result["entries_affected"], 2);
    assert_eq!(result["specs_affected"], 1);
    assert_eq!(
        result["results"][0]["spec_path"],
        "specs/history/history.spec.md"
    );
    assert_eq!(result["results"][0]["action"], "would_compact");
    assert_eq!(result["results"][0]["kept_entries"], 2);
    assert_eq!(fs::read(spec_path).unwrap(), before);
}

#[test]
fn compact_markdown_and_github_are_structured_and_preserve_dry_run() {
    let tmp = TempDir::new().unwrap();
    let root = setup_compact_project(&tmp);
    let spec_path = root.join("specs/history/history.spec.md");
    let before = fs::read(&spec_path).unwrap();

    for format in ["markdown", "github"] {
        specsync()
            .args(["compact", "--keep", "2", "--dry-run", "--format", format])
            .arg("--root")
            .arg(&root)
            .assert()
            .success()
            .stdout(predicate::str::contains("## SpecSync Compact Results"))
            .stdout(predicate::str::contains(
                "> Dry run — no files will be modified.",
            ))
            .stdout(predicate::str::contains(
                "| Spec | Action | Entries affected | Kept |",
            ))
            .stdout(predicate::str::contains(
                "| `specs/history/history.spec.md` | Would compact | 2 | 2 |",
            ))
            .stdout(predicate::str::contains(
                "**Summary:** Would compact 2 entries across 1 spec.",
            ));
    }

    assert_eq!(fs::read(spec_path).unwrap(), before);
}

#[test]
fn compact_parse_failure_exits_one_with_valid_json_and_zero_writes() {
    let tmp = TempDir::new().unwrap();
    let root = setup_compact_project(&tmp);
    let valid_path = root.join("specs/history/history.spec.md");
    let valid_before = fs::read(&valid_path).unwrap();

    fs::create_dir_all(root.join("src/broken")).unwrap();
    fs::write(
        root.join("src/broken/mod.ts"),
        "export function broken() {}\n",
    )
    .unwrap();
    fs::create_dir_all(root.join("specs/broken")).unwrap();
    let broken_path = root.join("specs/broken/broken.spec.md");
    let broken = valid_spec("broken", &["src/broken/mod.ts"])
        .replace("|------|--------|--------|", "|------|--------|");
    fs::write(&broken_path, &broken).unwrap();

    let output = specsync()
        .args(["compact", "--keep", "2", "--format", "json", "--root"])
        .arg(&root)
        .assert()
        .failure()
        .code(1)
        .get_output()
        .clone();
    let report: serde_json::Value =
        serde_json::from_slice(&output.stdout).expect("failure stdout must remain valid JSON");

    assert_eq!(report["complete"], false);
    assert_eq!(report["partial"], false);
    assert_eq!(report["applied"], false);
    assert_eq!(report["operations"]["planned"], 1);
    assert_eq!(report["operations"]["succeeded"], 0);
    assert_eq!(report["operations"]["failed"], 1);
    assert_eq!(report["errors"][0]["operation"], "parse");
    assert_eq!(fs::read(valid_path).unwrap(), valid_before);
    assert_eq!(fs::read_to_string(broken_path).unwrap(), broken);
}

#[cfg(unix)]
#[test]
fn compact_json_preserves_an_actual_unix_backslash_filename() {
    let tmp = TempDir::new().unwrap();
    let root = tmp.path();
    write_config(root, "specs", &["src"]);
    fs::create_dir_all(root.join("src/history")).unwrap();
    fs::write(
        root.join("src/history/mod.ts"),
        "export function record() {}\n",
    )
    .unwrap();
    let spec_dir = root.join(r"specs/history\literal");
    fs::create_dir_all(&spec_dir).unwrap();
    let mut spec = valid_spec("history", &["src/history/mod.ts"]);
    for day in 1..=3 {
        spec.push_str(&format!(
            "| 2026-07-{day:02} | maintainer | Change {day} |\n"
        ));
    }
    fs::write(spec_dir.join("history.spec.md"), spec).unwrap();

    let output = specsync()
        .args(["compact", "--keep", "1", "--dry-run", "--format", "json"])
        .arg("--root")
        .arg(root)
        .assert()
        .success()
        .get_output()
        .clone();
    let report: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();

    assert_eq!(
        report["results"][0]["spec_path"],
        r"specs/history\literal/history.spec.md"
    );
}

// ─── archive-tasks: text and structured output ───────────────────────

fn setup_archive_tasks_project(tmp: &TempDir) -> std::path::PathBuf {
    let root = tmp.path().to_path_buf();
    write_config(&root, "specs", &["src"]);
    fs::create_dir_all(root.join("src/work")).unwrap();
    fs::write(root.join("src/work/mod.ts"), "export function work() {}\n").unwrap();
    fs::create_dir_all(root.join("specs/work")).unwrap();
    fs::write(
        root.join("specs/work/work.spec.md"),
        valid_spec("work", &["src/work/mod.ts"]),
    )
    .unwrap();
    fs::write(
        root.join("specs/work/tasks.md"),
        "---\nspec: work.spec.md\n---\n\n## Tasks\n\n- [x] Done one\n- [ ] Still open\n- [X] Done two\n",
    )
    .unwrap();
    root
}

#[test]
fn archive_tasks_text_dry_run_reports_without_writing() {
    let tmp = TempDir::new().unwrap();
    let root = setup_archive_tasks_project(&tmp);
    let tasks_path = root.join("specs/work/tasks.md");
    let before = fs::read(&tasks_path).unwrap();

    specsync()
        .args(["archive-tasks", "--dry-run", "--root"])
        .arg(&root)
        .assert()
        .success()
        .stdout(predicate::str::contains(
            "Dry run — no files will be modified",
        ))
        .stdout(predicate::str::contains("would archive 2 tasks"))
        .stdout(predicate::str::contains(
            "Would archive 2 tasks across 1 file",
        ));

    assert_eq!(fs::read(tasks_path).unwrap(), before);
}

#[test]
fn archive_tasks_json_formats_are_clean_truthful_and_equivalent() {
    let tmp = TempDir::new().unwrap();
    let root = setup_archive_tasks_project(&tmp);
    let tasks_path = root.join("specs/work/tasks.md");
    let before = fs::read(&tasks_path).unwrap();
    let mut outputs = Vec::new();

    for format_args in [vec!["--format", "json"], vec!["--json"]] {
        let output = specsync()
            .args(["archive-tasks", "--dry-run"])
            .args(format_args)
            .arg("--root")
            .arg(&root)
            .assert()
            .success()
            .get_output()
            .clone();
        assert!(
            !output.stdout.contains(&0x1b),
            "JSON stdout contained an ANSI escape"
        );
        outputs.push(
            serde_json::from_slice::<serde_json::Value>(&output.stdout)
                .expect("archive-tasks JSON stdout must be one valid document"),
        );
    }

    assert_eq!(outputs[0], outputs[1], "--json must match --format json");
    let result = &outputs[0];
    assert_eq!(result["command"], "archive-tasks");
    assert_eq!(result["dry_run"], true);
    assert_eq!(result["would_change"], true);
    assert_eq!(result["applied"], false);
    assert_eq!(result["tasks_affected"], 2);
    assert_eq!(result["files_affected"], 1);
    assert_eq!(result["results"][0]["tasks_path"], "specs/work/tasks.md");
    assert_eq!(result["results"][0]["action"], "would_archive");
    assert_eq!(result["results"][0]["tasks_affected"], 2);
    assert_eq!(fs::read(tasks_path).unwrap(), before);
}

#[test]
fn archive_tasks_markdown_is_structured_and_preserves_dry_run() {
    let tmp = TempDir::new().unwrap();
    let root = setup_archive_tasks_project(&tmp);
    let tasks_path = root.join("specs/work/tasks.md");
    let before = fs::read(&tasks_path).unwrap();

    for format in ["markdown", "github"] {
        specsync()
            .args(["archive-tasks", "--dry-run", "--format", format, "--root"])
            .arg(&root)
            .assert()
            .success()
            .stdout(predicate::str::contains(
                "## SpecSync Archive Tasks Results",
            ))
            .stdout(predicate::str::contains(
                "> Dry run — no files will be modified.",
            ))
            .stdout(predicate::str::contains(
                "| Tasks file | Action | Tasks affected |",
            ))
            .stdout(predicate::str::contains(
                "| `specs/work/tasks.md` | Would archive | 2 |",
            ))
            .stdout(predicate::str::contains(
                "**Summary:** Would archive 2 tasks across 1 file.",
            ));
    }

    assert_eq!(fs::read(tasks_path).unwrap(), before);
}

#[test]
fn archive_tasks_apply_failure_exits_one_and_reports_zero_writes() {
    let tmp = TempDir::new().unwrap();
    let root = setup_archive_tasks_project(&tmp);
    let valid_tasks = root.join("specs/work/tasks.md");
    let valid_before = fs::read(&valid_tasks).unwrap();

    fs::create_dir_all(root.join("src/invalid")).unwrap();
    fs::write(
        root.join("src/invalid/mod.ts"),
        "export function invalid() {}\n",
    )
    .unwrap();
    fs::create_dir_all(root.join("specs/invalid")).unwrap();
    fs::write(
        root.join("specs/invalid/invalid.spec.md"),
        valid_spec("invalid", &["src/invalid/mod.ts"]),
    )
    .unwrap();
    let invalid_tasks = root.join("specs/invalid/tasks.md");
    fs::write(&invalid_tasks, b"## Tasks\n\n- [x] invalid \xFF\n").unwrap();
    let invalid_before = fs::read(&invalid_tasks).unwrap();

    let output = specsync()
        .args(["archive-tasks", "--format", "json", "--root"])
        .arg(&root)
        .assert()
        .failure()
        .code(1)
        .get_output()
        .clone();
    let report: serde_json::Value =
        serde_json::from_slice(&output.stdout).expect("failure stdout must remain valid JSON");

    assert_eq!(report["complete"], false);
    assert_eq!(report["partial"], false);
    assert_eq!(report["applied"], false);
    assert_eq!(report["files_planned"], 1);
    assert_eq!(report["files_succeeded"], 0);
    assert_eq!(report["failed"][0]["operation"], "read");
    assert_eq!(fs::read(valid_tasks).unwrap(), valid_before);
    assert_eq!(fs::read(invalid_tasks).unwrap(), invalid_before);
}

// ─── hooks install: claude-code-hook must not clobber user settings ──────

#[test]
fn hooks_install_claude_code_hook_preserves_user_settings() {
    // Regression (H3): `hooks install --claude-code-hook` used to overwrite the
    // whole `hooks` object in .claude/settings.json, destroying the user's own
    // hooks. It must deep-merge instead.
    let tmp = TempDir::new().unwrap();
    let root = tmp.path();
    fs::create_dir_all(root.join(".claude")).unwrap();
    fs::write(
        root.join(".claude/settings.json"),
        "{\n  \"permissions\": { \"allow\": [\"Bash(ls:*)\"] },\n  \"hooks\": {\n    \"PreToolUse\": [\n      { \"matcher\": \"Bash\", \"hooks\": [{ \"type\": \"command\", \"command\": \"audit.sh\" }] }\n    ]\n  }\n}\n",
    )
    .unwrap();

    specsync()
        .current_dir(root)
        .args(["hooks", "install", "--claude-code-hook"])
        .assert()
        .success();

    let parsed: serde_json::Value =
        serde_json::from_str(&fs::read_to_string(root.join(".claude/settings.json")).unwrap())
            .unwrap();
    assert_eq!(
        parsed["permissions"]["allow"][0], "Bash(ls:*)",
        "unrelated settings must survive"
    );
    assert_eq!(
        parsed["hooks"]["PreToolUse"][0]["hooks"][0]["command"], "audit.sh",
        "the user's own hooks must survive"
    );
    assert!(
        parsed["hooks"]["PostToolUse"][0]["hooks"][0]["command"]
            .as_str()
            .unwrap()
            .contains("specsync"),
        "specsync's hook must be added"
    );
}
