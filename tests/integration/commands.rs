use crate::helpers::*;
use assert_cmd::Command;
use predicates::prelude::*;
use std::fs;
use tempfile::TempDir;

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
        .success();

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
