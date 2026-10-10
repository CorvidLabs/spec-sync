//! Coverage reports, source input failures, and Gradle discovery gates.

use crate::helpers::*;
use predicates::prelude::*;
use std::fs;
use tempfile::TempDir;

#[path = "coverage/retained_root.rs"]
mod retained_root;

#[test]
fn malformed_gradle_is_inconclusive_for_coverage_gating_commands() {
    let tmp = TempDir::new().unwrap();
    let root = setup_minimal_project(&tmp);
    fs::write(root.join("build.gradle.kts"), "plugins {}\n").unwrap();
    fs::write(root.join("settings.gradle.kts"), "include(\":member\"\n").unwrap();
    omit_config_source_dirs(&root);

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

/// The other half of every refusal above (#723): the same unsafe Gradle input,
/// with `source_dirs` STATED.
///
/// Discovery still refuses these manifests. What changes is what the refusal is
/// allowed to conclude. Discovery exists to INFER a source list the project did
/// not state, so when the project HAS stated one, a failure to infer it is not a
/// verdict about the project and must not abort the command — which is why an
/// ordinary Gradle project could not run `check --strict` or `coverage` at all
/// on any 6.0 candidate, `source_dirs` configured or not.
///
/// Everything the refusal was actually protecting has to survive that, so this
/// asserts it directly rather than trusting the exit code to stand in for it: no
/// byte outside the root is read or disclosed, nothing is generated out of the
/// rejected discovery, and the outside tree is untouched. The gating commands
/// additionally DISCLOSE the degradation — manifest modules seed module
/// attribution, so a degraded run names fewer modules without specs than the
/// tree holds, and that is a report improved by a measurement that stopped.
#[test]
fn unsafe_gradle_discovery_degrades_over_stated_source_dirs_without_escaping() {
    for (label, settings, module, notice_fragment) in [
        (
            "malformed",
            "include(\":member\"\n".to_string(),
            "member",
            "unbalanced parentheses",
        ),
        (
            "root-escape",
            "include(\":outside\")\nproject(\":outside\").projectDir = file(\"../outside\")\n"
                .to_string(),
            "outside",
            "must remain beneath the project root",
        ),
        (
            "interpolated",
            "val outside = \"../outside\"\ninclude(\":member\")\nproject(\":member\").projectDir = file(\"$outside\")\n"
                .to_string(),
            "member",
            "Unsupported or dynamic Gradle expression",
        ),
        (
            "unsupported-mutator",
            "includeFlat(\"../outside\")\n".to_string(),
            "outside",
            "Unsupported Gradle workspace mutator",
        ),
    ] {
        let tmp = TempDir::new().unwrap();
        let root = tmp.path().join("project");
        let outside_source = tmp.path().join("outside/src/main/kotlin/Secret.kt");
        setup_minimal_project_at(&root);
        fs::create_dir_all(outside_source.parent().unwrap()).unwrap();
        let outside_bytes = format!("const val SECRET = \"DEGRADED_{label}\"\n");
        fs::write(&outside_source, outside_bytes.as_bytes()).unwrap();
        fs::write(root.join("settings.gradle.kts"), &settings).unwrap();

        for command in ["check", "coverage", "generate", "report", "score"] {
            let assertion = specsync()
                .arg(command)
                .arg("--root")
                .arg(&root)
                .args(["--format", "json"])
                .assert();
            // `report` and `score` can still exit 1 over something the manifest
            // has nothing to do with — these fixtures are not git repositories,
            // so staleness is unmeasurable. The claim under test is that the
            // MANIFEST no longer decides the outcome, so those two are judged on
            // the report they produced rather than on the status.
            let assertion = if matches!(command, "check" | "coverage" | "generate") {
                assertion.success()
            } else {
                assertion
            };
            let output = assertion.get_output().stdout.clone();
            let json: serde_json::Value = serde_json::from_slice(&output).unwrap_or_else(|error| {
                panic!(
                    "{command} must emit valid JSON after degraded Gradle {label} discovery: {error}; stdout={}",
                    String::from_utf8_lossy(&output)
                )
            });
            assert_ne!(
                json["inconclusive"],
                serde_json::json!(true),
                "{command} was still inconclusive over a stated source list for Gradle {label}: {json}"
            );
            match command {
                // The two gating commands carry the disclosure channel.
                "check" | "coverage" => {
                    let notices = json["manifest_notices"].as_array().unwrap_or_else(|| {
                        panic!("{command} must carry manifest_notices for Gradle {label}: {json}")
                    });
                    assert!(
                        notices.iter().any(|notice| notice
                            .as_str()
                            .is_some_and(|text| text.contains(notice_fragment))),
                        "unexpected {command} manifest_notices for Gradle {label}: {json}"
                    );
                }
                // The rest must have produced a real report rather than the
                // zeroed shape an inconclusive run emits.
                "report" => {
                    assert_eq!(json["overall_coverage_pct"], 100.0, "{json}");
                    assert_eq!(json["total_modules"], 1, "{json}");
                }
                "score" => assert_eq!(json["total_specs"], 1, "{json}"),
                _ => {}
            }
            assert!(
                !String::from_utf8_lossy(&output).contains("SECRET"),
                "{command} disclosed outside source bytes after degrading Gradle {label} discovery"
            );
            assert!(
                !root.join("specs").join(module).exists(),
                "{command} generated from a discovery it had rejected for Gradle {label}"
            );
            assert_eq!(
                fs::read(&outside_source).unwrap(),
                outside_bytes.as_bytes(),
                "{command} changed outside bytes after degrading Gradle {label} discovery"
            );
        }
    }
}

/// The reported project, end to end (#725): a composite build declared the way
/// composite builds are actually declared.
///
/// `includeBuild(path) { dependencySubstitution { … } }` is the reason to have a
/// composite build at all — it substitutes the local project for a published
/// coordinate — and the parser refused it while accepting the bare
/// `includeBuild(path)` minority form. The adopter saw the whole manifest
/// discarded over a block that declares no project, so `:app` went unreported
/// and the run fell back to whatever `source_dirs` happened to say.
///
/// This asserts the outcome the adopter cares about rather than the parser
/// verdict: the module is DISCOVERED, no degradation notice is emitted, and the
/// included build still contributes nothing — `vendor/shared` is a separate
/// build, so its sources must not be measured as part of this one.
#[test]
fn gradle_include_build_with_a_configuration_block_discovers_modules() {
    let tmp = TempDir::new().unwrap();
    let root = tmp.path().to_path_buf();
    fs::create_dir_all(root.join("app/src/main/kotlin")).unwrap();
    fs::create_dir_all(root.join("vendor/shared/src/main/kotlin")).unwrap();
    fs::create_dir_all(root.join("specs")).unwrap();
    write_config(&root, "specs", &["src"]);
    omit_config_source_dirs(&root);
    fs::write(root.join("build.gradle.kts"), "plugins {}\n").unwrap();
    fs::write(
        root.join("settings.gradle.kts"),
        "rootProject.name = \"podo\"\n\
         \n\
         includeBuild(\"vendor/shared\") {\n\
         \x20   dependencySubstitution {\n\
         \x20       substitute(module(\"com.example:podo\")).using(project(\":\"))\n\
         \x20   }\n\
         }\n\
         \n\
         include(\":app\")\n",
    )
    .unwrap();
    fs::write(root.join("app/src/main/kotlin/Main.kt"), "fun main() {}\n").unwrap();
    fs::write(
        root.join("vendor/shared/src/main/kotlin/Shared.kt"),
        "const val SEPARATE_BUILD = 1\n",
    )
    .unwrap();

    let output = specsync()
        .arg("coverage")
        .arg("--root")
        .arg(&root)
        .args(["--format", "json"])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    let json: serde_json::Value = serde_json::from_slice(&output).unwrap_or_else(|error| {
        panic!(
            "coverage must emit valid JSON for a composite build: {error}; stdout={}",
            String::from_utf8_lossy(&output)
        )
    });

    assert_ne!(json["inconclusive"], serde_json::json!(true), "{json}");
    assert_eq!(
        json["manifest_notices"],
        serde_json::json!([]),
        "coverage degraded over a composite build it can parse: {json}"
    );
    let modules = json["modules"]
        .as_array()
        .unwrap_or_else(|| panic!("coverage must report modules for a composite build: {json}"));
    assert!(
        modules
            .iter()
            .any(|module| module["name"] == serde_json::json!("app")),
        "the settings module was not discovered: {json}"
    );
    // The included build is a separate build: it contributes no module and no
    // source directory, so nothing beneath `vendor/shared` is measured here.
    assert_eq!(
        json["uncovered_files"],
        serde_json::json!([{ "file": "app/src/main/kotlin/Main.kt", "loc": 1 }]),
        "the included build was measured as part of the root build: {json}"
    );
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
    omit_config_source_dirs(&root);

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

/// Drop `sourceDirs` from a fixture config so the project's source list is the
/// output of the discovery under test rather than something the project stated.
///
/// The refusals below are about what an unsafe manifest may CONCLUDE, and that
/// depends on what the command was relying on discovery for. With no stated
/// `source_dirs` there is nothing trustworthy left to measure, so the refusal is
/// the whole answer and the command is inconclusive. `setup_minimal_project`
/// states `["src"]`, so these fixtures have to clear it to exercise that case
/// (#723); the stated case is asserted by
/// `unsafe_gradle_discovery_degrades_over_stated_source_dirs_without_escaping`.
fn omit_config_source_dirs(root: &std::path::Path) {
    let path = root.join("specsync.json");
    let config = fs::read_to_string(&path).unwrap();
    let mut value: serde_json::Value = serde_json::from_str(&config).unwrap();
    value
        .as_object_mut()
        .expect("fixture config must be a JSON object")
        .remove("sourceDirs");
    fs::write(&path, serde_json::to_string_pretty(&value).unwrap()).unwrap();
}

fn assert_gradle_discovery_is_inconclusive(
    root: &std::path::Path,
    outside_source: &std::path::Path,
    outside_bytes: &[u8],
    module: &str,
    label: &str,
) {
    omit_config_source_dirs(root);
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
