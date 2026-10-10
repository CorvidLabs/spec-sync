//! Issue discovery confines configured paths and refuses symlink/junction escapes.

use crate::helpers::*;
use std::fs;
use tempfile::TempDir;

#[cfg(windows)]
use super::super::create_windows_junction;

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
