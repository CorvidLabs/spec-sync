//! Deterministic generation, companion files, and structured-output gates.

use crate::helpers::*;
use predicates::prelude::*;
use std::fs;
use tempfile::TempDir;

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
fn generate_fails_closed_when_unspecced_modules_have_no_source_files() {
    let tmp = TempDir::new().unwrap();
    let root = tmp.path();
    fs::create_dir_all(root.join(".specsync")).unwrap();
    fs::create_dir_all(root.join("src")).unwrap();
    fs::write(
        root.join(".specsync/config.toml"),
        r#"
specs_dir = "specs"
source_dirs = ["src"]

[modules.ghost]
files = ["src/ghost.ts"]
"#,
    )
    .unwrap();

    specsync()
        .arg("generate")
        .arg("--root")
        .arg(root)
        .assert()
        .failure()
        .code(1)
        .stderr(predicate::str::contains("have no source files to bind"));

    specsync()
        .args(["generate", "--json", "--root"])
        .arg(root)
        .assert()
        .failure()
        .code(1)
        .stdout(predicate::str::contains("skipped_no_files"));
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
