use crate::helpers::*;
use predicates::prelude::*;
use std::fs;
use tempfile::TempDir;

fn complete_coverage_spec(module: &str, files: &[&str]) -> String {
    valid_spec(module, files)
        .replace(
            "| Condition | Behavior |\n|-----------|----------|",
            "| Condition | Behavior |\n|-----------|----------|\n| Invalid invocation | Exits non-zero without changing source files |",
        )
        .replace(
            "| Date | Author | Change |\n|------|--------|--------|",
            "| Date | Author | Change |\n|------|--------|--------|\n| 2026-07-14 | Integration test | Define the measured source fixture |",
        )
}

// ─── 1. specsync check ──────────────────────────────────────────────────

#[test]
fn check_valid_project_passes() {
    let tmp = TempDir::new().unwrap();
    let root = setup_minimal_project(&tmp);

    specsync()
        .arg("check")
        .arg("--root")
        .arg(&root)
        .assert()
        .success()
        .stdout(predicate::str::contains("specs checked"))
        .stdout(predicate::str::contains("0 failed"));
}

#[test]
fn sdd_failure_json_preserves_check_schema() {
    let tmp = TempDir::new().unwrap();
    let root = setup_minimal_project(&tmp);
    fs::create_dir_all(root.join(".specsync")).unwrap();
    fs::write(root.join(".specsync/sdd.json"), "{").unwrap();

    let assert = specsync()
        .arg("check")
        .arg("--root")
        .arg(&root)
        .arg("--format")
        .arg("json")
        .assert()
        .failure();
    let value: serde_json::Value =
        serde_json::from_slice(&assert.get_output().stdout).expect("single valid JSON result");
    for key in [
        "passed",
        "errors",
        "warnings",
        "stale",
        "specs_checked",
        "sdd",
    ] {
        assert!(value.get(key).is_some(), "missing JSON key {key}");
    }
    assert_eq!(value["passed"], false);
}

#[test]
fn check_missing_source_file_fails() {
    let tmp = TempDir::new().unwrap();
    let root = tmp.path().to_path_buf();

    write_config(&root, "specs", &["src"]);
    fs::create_dir_all(root.join("src/auth")).unwrap();
    // Do NOT create the source file referenced in the spec.
    fs::create_dir_all(root.join("specs/auth")).unwrap();
    let spec = valid_spec("auth", &["src/auth/missing.ts"]);
    fs::write(root.join("specs/auth/auth.spec.md"), spec).unwrap();

    specsync()
        .arg("check")
        .arg("--root")
        .arg(&root)
        .arg("--enforcement")
        .arg("strict")
        .assert()
        .failure()
        .stdout(predicate::str::contains("Source file not found"));
}

#[test]
fn draft_planned_mapping_passes_strict_and_is_absent_from_coverage() {
    let tmp = TempDir::new().unwrap();
    let root = tmp.path();
    write_config(root, "specs", &["src"]);
    fs::create_dir_all(root.join("src")).unwrap();
    fs::create_dir_all(root.join("specs/current")).unwrap();
    fs::create_dir_all(root.join("specs/future")).unwrap();
    fs::write(root.join("src/current.ts"), "// current\n").unwrap();
    fs::write(
        root.join("specs/current/current.spec.md"),
        complete_coverage_spec("current", &["src/current.ts"]),
    )
    .unwrap();
    fs::write(
        root.join("specs/future/future.spec.md"),
        complete_coverage_spec("future", &["src/future.ts"])
            .replace("status: active", "status: draft"),
    )
    .unwrap();

    specsync()
        .args(["check", "--strict", "--require-coverage", "100", "--force"])
        .arg("--root")
        .arg(root)
        .assert()
        .success()
        .stdout(predicate::str::contains(
            "Planned source mapping (draft; file not created yet): src/future.ts",
        ))
        .stdout(predicate::str::contains("File coverage: 1/1 (100%)"))
        .stdout(predicate::str::contains("LOC coverage:  1/1 (100%)"));

    let json = specsync()
        .args(["check", "--strict", "--force", "--format", "json"])
        .arg("--root")
        .arg(root)
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    let value: serde_json::Value = serde_json::from_slice(&json).unwrap();
    assert_eq!(value["passed"], true);
    assert_eq!(value["warnings"], serde_json::json!([]));
    assert!(
        value["notices"][0]
            .as_str()
            .unwrap()
            .contains("src/future.ts")
    );

    for format in ["markdown", "github"] {
        specsync()
            .args(["check", "--strict", "--force", "--format", format])
            .arg("--root")
            .arg(root)
            .assert()
            .success()
            .stdout(predicate::str::contains("### Planned Mappings"))
            .stdout(predicate::str::contains("src/future.ts"));
    }

    fs::write(root.join("src/future.ts"), "// now implemented\n").unwrap();
    specsync()
        .args(["check", "--strict", "--require-coverage", "100", "--force"])
        .arg("--root")
        .arg(root)
        .assert()
        .success()
        .stdout(predicate::str::contains("File coverage: 2/2 (100%)"))
        .stdout(predicate::str::contains("LOC coverage:  2/2 (100%)"))
        .stdout(predicate::str::contains("Planned source mapping").not());
}

#[test]
fn mixed_draft_and_active_missing_mappings_only_exempt_the_draft() {
    let tmp = TempDir::new().unwrap();
    let root = tmp.path();
    write_config(root, "specs", &["src"]);
    fs::create_dir_all(root.join("specs/draft")).unwrap();
    fs::create_dir_all(root.join("specs/active")).unwrap();
    fs::write(
        root.join("specs/draft/draft.spec.md"),
        complete_coverage_spec("draft", &["src/planned.ts"])
            .replace("status: active", "status: draft"),
    )
    .unwrap();
    fs::write(
        root.join("specs/active/active.spec.md"),
        complete_coverage_spec("active", &["src/missing.ts"]),
    )
    .unwrap();

    specsync()
        .args(["check", "--strict", "--force"])
        .arg("--root")
        .arg(root)
        .assert()
        .failure()
        .stdout(predicate::str::contains("Planned source mapping"))
        .stdout(predicate::str::contains(
            "Source file not found: src/missing.ts",
        ));
}

#[test]
fn draft_mapping_transitions_on_activation_and_file_creation() {
    let tmp = TempDir::new().unwrap();
    let root = tmp.path();
    write_config(root, "specs", &["src"]);
    fs::create_dir_all(root.join("specs/future")).unwrap();
    let spec_path = root.join("specs/future/future.spec.md");
    let active = complete_coverage_spec("future", &["src/future.ts"]);
    fs::write(
        &spec_path,
        active.replace("status: active", "status: draft"),
    )
    .unwrap();

    specsync()
        .args(["check", "--strict", "--force"])
        .arg("--root")
        .arg(root)
        .assert()
        .success()
        .stdout(predicate::str::contains("Planned source mapping"));

    fs::write(&spec_path, &active).unwrap();
    specsync()
        .args(["check", "--strict", "--force"])
        .arg("--root")
        .arg(root)
        .assert()
        .failure()
        .stdout(predicate::str::contains("Source file not found"));

    fs::create_dir_all(root.join("src")).unwrap();
    fs::write(root.join("src/future.ts"), "// implemented\n").unwrap();
    specsync()
        .args(["check", "--strict", "--require-coverage", "100", "--force"])
        .arg("--root")
        .arg(root)
        .assert()
        .success()
        .stdout(predicate::str::contains("File coverage: 1/1 (100%)"))
        .stdout(predicate::str::contains("Planned source mapping").not());
}

#[test]
fn require_draft_files_restores_missing_file_errors() {
    let tmp = TempDir::new().unwrap();
    let root = tmp.path();
    fs::write(
        root.join(".specsync.toml"),
        "specs_dir = \"specs\"\nsource_dirs = [\"src\"]\nrequire_draft_files = true\n",
    )
    .unwrap();
    fs::create_dir_all(root.join("specs/future")).unwrap();
    fs::write(
        root.join("specs/future/future.spec.md"),
        complete_coverage_spec("future", &["src/future.ts"])
            .replace("status: active", "status: draft"),
    )
    .unwrap();

    specsync()
        .args(["check", "--strict", "--force"])
        .arg("--root")
        .arg(root)
        .assert()
        .failure()
        .stdout(predicate::str::contains(
            "Source file not found: src/future.ts",
        ));
}

#[test]
fn draft_existing_files_keep_ownership_and_path_safety_validation() {
    let tmp = TempDir::new().unwrap();
    let root = tmp.path();
    write_config(root, "specs", &["src"]);
    fs::create_dir_all(root.join("src")).unwrap();
    fs::create_dir_all(root.join("specs/one")).unwrap();
    fs::create_dir_all(root.join("specs/two")).unwrap();
    fs::write(root.join("src/shared.ts"), "// shared\n").unwrap();
    fs::write(
        root.join("specs/one/one.spec.md"),
        complete_coverage_spec("one", &["src/shared.ts"])
            .replace("status: active", "status: draft"),
    )
    .unwrap();
    fs::write(
        root.join("specs/two/two.spec.md"),
        complete_coverage_spec("two", &["src/shared.ts"]),
    )
    .unwrap();

    specsync()
        .args(["check", "--strict", "--force"])
        .arg("--root")
        .arg(root)
        .assert()
        .failure()
        .stdout(predicate::str::contains(
            "Source file has duplicate spec ownership: src/shared.ts",
        ));

    fs::write(
        root.join("specs/one/one.spec.md"),
        complete_coverage_spec("one", &["../outside.ts"])
            .replace("status: active", "status: draft"),
    )
    .unwrap();
    fs::remove_file(root.join("specs/two/two.spec.md")).unwrap();
    specsync()
        .args(["check", "--strict", "--force"])
        .arg("--root")
        .arg(root)
        .assert()
        .failure()
        .stdout(predicate::str::contains(
            "Source file not found: ../outside.ts",
        ))
        .stdout(predicate::str::contains("Planned source mapping").not());
}

#[test]
fn incremental_check_detects_duplicate_ownership_against_cached_specs() {
    let tmp = TempDir::new().unwrap();
    let root = tmp.path();
    write_config(root, "specs", &["src"]);
    fs::create_dir_all(root.join("src")).unwrap();
    fs::create_dir_all(root.join("specs/one")).unwrap();
    fs::create_dir_all(root.join("specs/two")).unwrap();
    fs::write(root.join("src/shared.ts"), "// shared\n").unwrap();
    fs::write(root.join("src/other.ts"), "// other\n").unwrap();
    fs::write(
        root.join("specs/one/one.spec.md"),
        complete_coverage_spec("one", &["src/shared.ts"]),
    )
    .unwrap();
    let second_spec = root.join("specs/two/two.spec.md");
    fs::write(
        &second_spec,
        complete_coverage_spec("two", &["src/other.ts"]).replace("status: active", "status: draft"),
    )
    .unwrap();

    specsync()
        .arg("check")
        .arg("--root")
        .arg(root)
        .assert()
        .success();

    fs::write(
        &second_spec,
        complete_coverage_spec("two", &["src/shared.ts"])
            .replace("status: active", "status: draft"),
    )
    .unwrap();

    specsync()
        .arg("check")
        .args(["--enforcement", "strict"])
        .arg("--root")
        .arg(root)
        .assert()
        .failure()
        .stdout(predicate::str::contains(
            "Source file has duplicate spec ownership: src/shared.ts",
        ));
}

#[test]
fn invalid_existing_mapping_is_not_tracked_as_duplicate_ownership() {
    let tmp = TempDir::new().unwrap();
    let root = tmp.path();
    write_config(root, "specs", &["src"]);
    fs::create_dir_all(root.join("src")).unwrap();
    fs::create_dir_all(root.join("specs/one")).unwrap();
    fs::create_dir_all(root.join("specs/two")).unwrap();
    let shared = root.join("src/shared.ts");
    fs::write(&shared, "// shared\n").unwrap();
    fs::write(
        root.join("specs/one/one.spec.md"),
        complete_coverage_spec("one", &[shared.to_string_lossy().as_ref()]),
    )
    .unwrap();
    fs::write(
        root.join("specs/two/two.spec.md"),
        complete_coverage_spec("two", &["src/shared.ts"]),
    )
    .unwrap();

    specsync()
        .args(["check", "--strict", "--force"])
        .arg("--root")
        .arg(root)
        .assert()
        .success()
        .stdout(predicate::str::contains("Source file has duplicate spec ownership").not());
}

#[test]
fn draft_dot_segment_mapping_transitions_to_covered_file() {
    let tmp = TempDir::new().unwrap();
    let root = tmp.path();
    write_config(root, "specs", &["src"]);
    fs::create_dir_all(root.join("src")).unwrap();
    fs::create_dir_all(root.join("specs/current")).unwrap();
    fs::create_dir_all(root.join("specs/future")).unwrap();
    fs::write(root.join("src/current.ts"), "// current\n").unwrap();
    fs::write(
        root.join("specs/current/current.spec.md"),
        complete_coverage_spec("current", &["src/current.ts"]),
    )
    .unwrap();
    fs::write(
        root.join("specs/future/future.spec.md"),
        complete_coverage_spec("future", &["./src/future.ts"])
            .replace("status: active", "status: draft"),
    )
    .unwrap();

    specsync()
        .args(["check", "--strict", "--require-coverage", "100", "--force"])
        .arg("--root")
        .arg(root)
        .assert()
        .success()
        .stdout(predicate::str::contains("File coverage: 1/1 (100%)"))
        .stdout(predicate::str::contains("Planned source mapping"));

    fs::write(root.join("src/future.ts"), "// implemented\n").unwrap();
    specsync()
        .args(["check", "--strict", "--require-coverage", "100", "--force"])
        .arg("--root")
        .arg(root)
        .assert()
        .success()
        .stdout(predicate::str::contains("File coverage: 2/2 (100%)"))
        .stdout(predicate::str::contains("LOC coverage:  2/2 (100%)"))
        .stdout(predicate::str::contains("Planned source mapping").not());
}

#[test]
fn check_undocumented_export_warns() {
    let tmp = TempDir::new().unwrap();
    let root = tmp.path().to_path_buf();

    write_config(&root, "specs", &["src"]);

    fs::create_dir_all(root.join("src/utils")).unwrap();
    fs::write(
        root.join("src/utils/helpers.ts"),
        "export function documented() {}\nexport function undocumented() {}\n",
    )
    .unwrap();

    fs::create_dir_all(root.join("specs/utils")).unwrap();
    // Spec only documents `documented`, not `undocumented`
    let spec = r#"---
module: utils
version: 1
status: active
files:
  - src/utils/helpers.ts
db_tables: []
depends_on: []
---

# Utils

## Purpose

Utility functions.

## Public API

### Exported Functions

| Function | Parameters | Returns | Description |
|----------|-----------|---------|-------------|
| `documented` | none | void | Does something |

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

### Consumes

| Module | What is used |
|--------|-------------|

### Consumed By

| Module | What is used |
|--------|-------------|

## Change Log

| Date | Author | Change |
|------|--------|--------|
"#;
    fs::write(root.join("specs/utils/utils.spec.md"), spec).unwrap();

    specsync()
        .arg("check")
        .arg("--root")
        .arg(&root)
        .assert()
        .success()
        .stdout(predicate::str::contains(
            "Undocumented export 'undocumented' from src/utils/helpers.ts",
        ));
}

#[test]
fn check_phantom_export_errors() {
    let tmp = TempDir::new().unwrap();
    let root = tmp.path().to_path_buf();

    write_config(&root, "specs", &["src"]);

    fs::create_dir_all(root.join("src/core")).unwrap();
    fs::write(
        root.join("src/core/engine.ts"),
        "export function realExport() {}\n",
    )
    .unwrap();

    fs::create_dir_all(root.join("specs/core")).unwrap();
    // Spec documents `phantomExport` which does not exist in source
    let spec = r#"---
module: core
version: 1
status: active
files:
  - src/core/engine.ts
db_tables: []
depends_on: []
---

# Core

## Purpose

Core engine.

## Public API

### Exported Functions

| Function | Parameters | Returns | Description |
|----------|-----------|---------|-------------|
| `phantomExport` | none | void | Does not exist |

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

### Consumes

| Module | What is used |
|--------|-------------|

### Consumed By

| Module | What is used |
|--------|-------------|

## Change Log

| Date | Author | Change |
|------|--------|--------|
"#;
    fs::write(root.join("specs/core/core.spec.md"), spec).unwrap();

    specsync()
        .arg("check")
        .arg("--root")
        .arg(&root)
        .arg("--enforcement")
        .arg("strict")
        .assert()
        .failure()
        .stdout(predicate::str::contains(
            "Spec documents 'phantomExport' but no matching export found",
        ));
}

// ─── 5. --strict flag ───────────────────────────────────────────────────

#[test]
fn strict_turns_warnings_into_errors() {
    let tmp = TempDir::new().unwrap();
    let root = tmp.path().to_path_buf();

    write_config(&root, "specs", &["src"]);

    fs::create_dir_all(root.join("src/svc")).unwrap();
    fs::write(
        root.join("src/svc/api.ts"),
        "export function documented() {}\nexport function extra() {}\n",
    )
    .unwrap();

    fs::create_dir_all(root.join("specs/svc")).unwrap();
    // Only document one of two exports -> warning for undocumented
    let spec = r#"---
module: svc
version: 1
status: active
files:
  - src/svc/api.ts
db_tables: []
depends_on: []
---

# Svc

## Purpose

Service.

## Public API

### Exported Functions

| Function | Parameters | Returns | Description |
|----------|-----------|---------|-------------|
| `documented` | none | void | Documented |

## Invariants

1. Valid.

## Behavioral Examples

### Scenario: Basic

- **Given** precondition
- **When** action
- **Then** result

## Error Cases

| Condition | Behavior |
|-----------|----------|

## Dependencies

### Consumes

| Module | What is used |
|--------|-------------|

### Consumed By

| Module | What is used |
|--------|-------------|

## Change Log

| Date | Author | Change |
|------|--------|--------|
"#;
    fs::write(root.join("specs/svc/svc.spec.md"), spec).unwrap();

    // Without --strict: passes (warnings only)
    specsync()
        .arg("check")
        .arg("--root")
        .arg(&root)
        .assert()
        .success();

    // With --strict: fails
    specsync()
        .arg("check")
        .arg("--strict")
        .arg("--root")
        .arg(&root)
        .assert()
        .failure()
        .stdout(predicate::str::contains("--strict mode"));
}

#[test]
fn strict_rejects_unfilled_companion_scaffold_markers() {
    let tmp = TempDir::new().unwrap();
    let root = setup_minimal_project(&tmp);
    fs::write(
        root.join("specs/auth/context.md"),
        "# Context\n\n<!-- Describe the context and motivation for this module. -->\n",
    )
    .unwrap();

    specsync()
        .arg("check")
        .arg("--root")
        .arg(&root)
        .assert()
        .success()
        .stdout(predicate::str::contains(
            "Unfilled companion scaffold marker at specs/auth/context.md:3",
        ));

    specsync()
        .arg("check")
        .arg("--strict")
        .arg("--root")
        .arg(&root)
        .assert()
        .failure()
        .stdout(predicate::str::contains(
            "Unfilled companion scaffold marker at specs/auth/context.md:3",
        ));
}

// ─── 6. --require-coverage flag ─────────────────────────────────────────

#[test]
fn require_coverage_passes_when_met() {
    let tmp = TempDir::new().unwrap();
    let root = setup_minimal_project(&tmp);

    specsync()
        .arg("check")
        .arg("--require-coverage")
        .arg("100")
        .arg("--root")
        .arg(&root)
        .assert()
        .success();
}

#[test]
fn html_static_content_is_measured_by_strict_coverage() {
    let tmp = TempDir::new().unwrap();
    let root = tmp.path().to_path_buf();
    write_config(&root, "specs", &["landing"]);
    fs::create_dir_all(root.join("landing")).unwrap();
    fs::create_dir_all(root.join("specs/landing")).unwrap();
    fs::write(root.join("landing/index.html"), "<main>Welcome</main>\n").unwrap();
    let unmapped = valid_spec("landing", &[]).replace("files:\ndb_tables", "files: []\ndb_tables");
    let spec_path = root.join("specs/landing/landing.spec.md");
    fs::write(&spec_path, unmapped).unwrap();

    specsync()
        .arg("check")
        .arg("--require-coverage")
        .arg("100")
        .arg("--root")
        .arg(&root)
        .assert()
        .failure()
        .stdout(predicate::str::contains("File coverage: 0/1"));

    fs::write(&spec_path, valid_spec("landing", &["landing/index.html"])).unwrap();
    specsync()
        .arg("check")
        .arg("--require-coverage")
        .arg("100")
        .arg("--root")
        .arg(&root)
        .assert()
        .success()
        .stdout(predicate::str::contains("File coverage: 1/1"));
}

#[test]
fn extensionless_only_project_has_non_vacuous_strict_coverage() {
    let tmp = TempDir::new().unwrap();
    let root = tmp.path();
    fs::create_dir_all(root.join(".specsync")).unwrap();
    fs::create_dir_all(root.join("bin")).unwrap();
    fs::create_dir_all(root.join("specs/tool")).unwrap();
    fs::write(
        root.join(".specsync/config.toml"),
        "specs_dir = \"specs\"\nsource_dirs = [\"bin\"]\nsource_extensions = []\ninclude_extensionless = true\n",
    )
    .unwrap();
    fs::write(root.join("bin/tool"), "#!/bin/sh\necho tool\n").unwrap();
    fs::write(
        root.join("specs/tool/tool.spec.md"),
        complete_coverage_spec("tool", &["bin/tool"]),
    )
    .unwrap();

    specsync()
        .args(["coverage", "--strict", "--require-coverage", "100"])
        .arg("--root")
        .arg(root)
        .assert()
        .success()
        .stdout(predicate::str::contains("File coverage: 1/1 (100%)"))
        .stdout(predicate::str::contains("LOC coverage:  2/2 (100%)"));
}

#[test]
fn mixed_extensionless_project_has_non_vacuous_strict_coverage() {
    let tmp = TempDir::new().unwrap();
    let root = tmp.path();
    fs::create_dir_all(root.join(".specsync")).unwrap();
    fs::create_dir_all(root.join("bin")).unwrap();
    fs::create_dir_all(root.join("specs/tool")).unwrap();
    fs::create_dir_all(root.join("specs/helper")).unwrap();
    fs::write(
        root.join(".specsync/config.toml"),
        "specs_dir = \"specs\"\nsource_dirs = [\"bin\"]\nsource_extensions = [\"sh\"]\ninclude_extensionless = true\n",
    )
    .unwrap();
    fs::write(root.join("bin/tool"), "#!/bin/sh\necho tool\n").unwrap();
    fs::write(root.join("bin/helper.sh"), "#!/bin/sh\necho helper\n").unwrap();
    fs::write(
        root.join("specs/tool/tool.spec.md"),
        complete_coverage_spec("tool", &["bin/tool"]),
    )
    .unwrap();
    fs::write(
        root.join("specs/helper/helper.spec.md"),
        complete_coverage_spec("helper", &["bin/helper.sh"]),
    )
    .unwrap();

    specsync()
        .args(["coverage", "--strict", "--require-coverage", "100"])
        .arg("--root")
        .arg(root)
        .assert()
        .success()
        .stdout(predicate::str::contains("File coverage: 2/2 (100%)"))
        .stdout(predicate::str::contains("LOC coverage:  4/4 (100%)"));
}

#[test]
fn default_discovery_counts_mjs_and_cjs_in_exact_coverage_totals() {
    let tmp = TempDir::new().unwrap();
    let root = tmp.path();
    fs::create_dir_all(root.join("src")).unwrap();
    fs::create_dir_all(root.join("specs/modules")).unwrap();
    write_config(root, "specs", &["src"]);
    fs::write(root.join("src/index.ts"), "// ts one\n// ts two\n").unwrap();
    fs::write(root.join("src/render.mjs"), "// mjs one\n// mjs two\n").unwrap();
    fs::write(root.join("src/config.cjs"), "// cjs one\n// cjs two\n").unwrap();
    fs::write(root.join("src/theme.css"), "/* css one */\n/* css two */\n").unwrap();
    fs::write(
        root.join("specs/modules/modules.spec.md"),
        complete_coverage_spec(
            "modules",
            &[
                "src/index.ts",
                "src/render.mjs",
                "src/config.cjs",
                "src/theme.css",
            ],
        ),
    )
    .unwrap();

    specsync()
        .args(["coverage", "--strict", "--require-coverage", "100"])
        .arg("--root")
        .arg(root)
        .assert()
        .success()
        .stdout(predicate::str::contains("File coverage: 4/4 (100%)"))
        .stdout(predicate::str::contains("LOC coverage:  8/8 (100%)"));
}

#[test]
fn uncovered_mjs_and_cjs_files_fail_strict_full_coverage() {
    for extension in ["mjs", "cjs"] {
        let tmp = TempDir::new().unwrap();
        let root = tmp.path();
        fs::create_dir_all(root.join("src")).unwrap();
        fs::create_dir_all(root.join("specs/index")).unwrap();
        write_config(root, "specs", &["src"]);
        fs::write(root.join("src/index.ts"), "// mapped\n").unwrap();
        fs::write(
            root.join(format!("src/unmapped.{extension}")),
            "// uncovered\n",
        )
        .unwrap();
        fs::write(
            root.join("specs/index/index.spec.md"),
            complete_coverage_spec("index", &["src/index.ts"]),
        )
        .unwrap();

        specsync()
            .args(["coverage", "--strict", "--require-coverage", "100"])
            .arg("--root")
            .arg(root)
            .assert()
            .failure()
            .stdout(predicate::str::contains("File coverage: 1/2 (50%)"))
            .stdout(predicate::str::contains("LOC coverage:  1/2 (50%)"));
    }
}

#[test]
fn javascript_family_test_files_do_not_inflate_default_coverage_totals() {
    let tmp = TempDir::new().unwrap();
    let root = tmp.path();
    fs::create_dir_all(root.join("src")).unwrap();
    fs::create_dir_all(root.join("specs/index")).unwrap();
    write_config(root, "specs", &["src"]);
    fs::write(root.join("src/index.ts"), "// mapped\n").unwrap();
    for filename in [
        "index.test.js",
        "index.spec.jsx",
        "index.test.mjs",
        "index.spec.cjs",
    ] {
        fs::write(root.join("src").join(filename), "// test-only\n").unwrap();
    }
    fs::write(
        root.join("specs/index/index.spec.md"),
        complete_coverage_spec("index", &["src/index.ts"]),
    )
    .unwrap();

    specsync()
        .args(["coverage", "--strict", "--require-coverage", "100"])
        .arg("--root")
        .arg(root)
        .assert()
        .success()
        .stdout(predicate::str::contains("File coverage: 1/1 (100%)"))
        .stdout(predicate::str::contains("LOC coverage:  1/1 (100%)"));
}

#[test]
fn extensionless_mjs_barrel_passes_strict_in_regex_and_ast_modes() {
    let tmp = TempDir::new().unwrap();
    let root = tmp.path();
    fs::create_dir_all(root.join("src")).unwrap();
    fs::create_dir_all(root.join("specs/modules")).unwrap();
    write_config(root, "specs", &["src"]);
    fs::write(
        root.join("src/values.mjs"),
        "export const fromMjs = true;\n",
    )
    .unwrap();
    fs::write(root.join("src/index.mjs"), "export * from './values';\n").unwrap();
    let spec = complete_coverage_spec("modules", &["src/values.mjs", "src/index.mjs"]).replace(
        "| Function | Parameters | Returns | Description |\n|----------|-----------|---------|-------------|",
        "| Function | Parameters | Returns | Description |\n|----------|-----------|---------|-------------|\n| `fromMjs` | none | boolean | Re-exported mjs value |",
    );
    fs::write(root.join("specs/modules/modules.spec.md"), spec).unwrap();

    for parse_mode in ["regex", "ast"] {
        fs::create_dir_all(root.join(".specsync")).unwrap();
        fs::write(
            root.join(".specsync/config.toml"),
            format!(
                "specs_dir = \"specs\"\nsource_dirs = [\"src\"]\nparse_mode = \"{parse_mode}\"\n"
            ),
        )
        .unwrap();
        specsync()
            .args(["check", "--strict", "--force"])
            .arg("--root")
            .arg(root)
            .assert()
            .success()
            .stdout(predicate::str::contains("1 specs checked: 1 passed"));
    }
}

#[test]
fn require_coverage_fails_when_below_threshold() {
    let tmp = TempDir::new().unwrap();
    let root = setup_minimal_project(&tmp);

    // Add uncovered file
    fs::write(
        root.join("src/auth/uncovered.ts"),
        "export function x() {}\n",
    )
    .unwrap();

    specsync()
        .arg("check")
        .arg("--require-coverage")
        .arg("100")
        .arg("--root")
        .arg(&root)
        .assert()
        .failure()
        .stdout(predicate::str::contains("--require-coverage"));
}

#[test]
fn require_coverage_fails_loud_on_zero_source_files() {
    // A `--require-coverage` gate over 0 source files reports a vacuous 100% and must
    // NOT silently pass. Here `**/**` excludes every source file; enforcement=warn so
    // ordinary spec diagnostics never exit 1, leaving the require-coverage gate as the
    // sole failure.
    let tmp = TempDir::new().unwrap();
    let root = tmp.path();
    fs::create_dir_all(root.join(".specsync")).unwrap();
    fs::write(
        root.join(".specsync/config.toml"),
        "specs_dir = \"specs\"\nsource_dirs = [\"src\"]\nexclude_patterns = [\"**/**\"]\nenforcement = \"warn\"\n",
    )
    .unwrap();
    fs::create_dir_all(root.join("src")).unwrap();
    fs::write(root.join("src/a.ts"), "export function a() {}\n").unwrap();
    fs::create_dir_all(root.join("specs/a")).unwrap();
    fs::write(
        root.join("specs/a/a.spec.md"),
        valid_spec("a", &["src/a.ts"]),
    )
    .unwrap();

    specsync()
        .arg("check")
        .arg("--require-coverage")
        .arg("100")
        .arg("--root")
        .arg(root)
        .assert()
        .failure()
        .stdout(predicate::str::contains("no source files were found"));
}

// ─── 7. --root flag ────────────────────────────────────────────────────

#[test]
fn root_flag_overrides_cwd() {
    let tmp = TempDir::new().unwrap();
    let root = setup_minimal_project(&tmp);

    // Run from a different directory but point --root at our project
    specsync()
        .arg("check")
        .arg("--root")
        .arg(&root)
        .current_dir(std::env::temp_dir())
        .assert()
        .success()
        .stdout(predicate::str::contains("specs checked"));
}

// ─── 9. Error cases ────────────────────────────────────────────────────

#[test]
fn no_spec_files_exits_cleanly() {
    let tmp = TempDir::new().unwrap();
    let root = tmp.path();

    write_config(root, "specs", &["src"]);
    fs::create_dir_all(root.join("specs")).unwrap();
    fs::create_dir_all(root.join("src")).unwrap();

    specsync()
        .arg("check")
        .arg("--root")
        .arg(root)
        .assert()
        .success()
        .stdout(predicate::str::contains("No spec files found"));
}

#[test]
fn invalid_frontmatter_reports_error() {
    let tmp = TempDir::new().unwrap();
    let root = tmp.path().to_path_buf();

    write_config(&root, "specs", &["src"]);
    fs::create_dir_all(root.join("src/bad")).unwrap();
    fs::write(root.join("src/bad/code.ts"), "export function x() {}\n").unwrap();

    fs::create_dir_all(root.join("specs/bad")).unwrap();
    // Spec with NO frontmatter at all
    fs::write(
        root.join("specs/bad/bad.spec.md"),
        "# No Frontmatter\n\nJust markdown, no YAML block.\n",
    )
    .unwrap();

    specsync()
        .arg("check")
        .arg("--root")
        .arg(&root)
        .arg("--enforcement")
        .arg("strict")
        .assert()
        .failure()
        .stdout(predicate::str::contains("0 passed"));
}

#[test]
fn missing_spec_dir_exits_cleanly() {
    let tmp = TempDir::new().unwrap();
    let root = tmp.path();

    write_config(root, "specs", &["src"]);
    // Do NOT create specs/ or src/ directories
    fs::create_dir_all(root.join("src")).unwrap();

    specsync()
        .arg("check")
        .arg("--root")
        .arg(root)
        .assert()
        .success()
        .stdout(predicate::str::contains("No spec files found"));
}

#[test]
fn missing_required_sections_reports_error() {
    let tmp = TempDir::new().unwrap();
    let root = tmp.path().to_path_buf();

    write_config(&root, "specs", &["src"]);
    fs::create_dir_all(root.join("src/partial")).unwrap();
    fs::write(root.join("src/partial/mod.ts"), "export function f() {}\n").unwrap();

    fs::create_dir_all(root.join("specs/partial")).unwrap();
    // Spec with frontmatter but missing most required sections
    let spec = r#"---
module: partial
version: 1
status: active
files:
  - src/partial/mod.ts
db_tables: []
depends_on: []
---

# Partial

## Purpose

Only has Purpose section.
"#;
    fs::write(root.join("specs/partial/partial.spec.md"), spec).unwrap();

    specsync()
        .arg("check")
        .arg("--root")
        .arg(&root)
        .arg("--enforcement")
        .arg("strict")
        .assert()
        .failure()
        .stdout(predicate::str::contains("Missing required section"));
}

#[test]
fn missing_frontmatter_fields_reports_error() {
    let tmp = TempDir::new().unwrap();
    let root = tmp.path().to_path_buf();

    write_config(&root, "specs", &["src"]);
    fs::create_dir_all(root.join("src/empty")).unwrap();
    fs::write(root.join("src/empty/mod.ts"), "export function f() {}\n").unwrap();

    fs::create_dir_all(root.join("specs/empty")).unwrap();
    // Frontmatter has delimiters but no fields
    let spec = r#"---
module: empty
---

# Empty

## Purpose

Something

## Public API

Nothing

## Invariants

1. Ok

## Behavioral Examples

### Scenario: Basic

- **Given** x
- **When** y
- **Then** z

## Error Cases

| Condition | Behavior |
|-----------|----------|

## Dependencies

None

## Change Log

| Date | Author | Change |
|------|--------|--------|
"#;
    fs::write(root.join("specs/empty/empty.spec.md"), spec).unwrap();

    specsync()
        .arg("check")
        .arg("--root")
        .arg(&root)
        .arg("--enforcement")
        .arg("strict")
        .assert()
        .failure()
        .stdout(predicate::str::contains("0 passed"))
        .stdout(predicate::str::contains("1 failed"));
}

#[test]
fn default_command_is_check() {
    let tmp = TempDir::new().unwrap();
    let root = setup_minimal_project(&tmp);

    // No subcommand specified -- should default to check
    specsync()
        .arg("--root")
        .arg(&root)
        .assert()
        .success()
        .stdout(predicate::str::contains("specs checked"));
}

#[test]
fn dependency_spec_not_found_errors() {
    let tmp = TempDir::new().unwrap();
    let root = tmp.path().to_path_buf();

    write_config(&root, "specs", &["src"]);

    fs::create_dir_all(root.join("src/dep")).unwrap();
    fs::write(root.join("src/dep/mod.ts"), "export function f() {}\n").unwrap();

    fs::create_dir_all(root.join("specs/dep")).unwrap();
    // depends_on references a spec that does not exist
    let spec = r#"---
module: dep
version: 1
status: active
files:
  - src/dep/mod.ts
db_tables: []
depends_on:
  - specs/nonexistent/nonexistent.spec.md
---

# Dep

## Purpose

Something.

## Public API

### Exported Functions

| Function | Parameters | Returns | Description |
|----------|-----------|---------|-------------|

## Invariants

1. Ok.

## Behavioral Examples

### Scenario: Basic

- **Given** precondition
- **When** action
- **Then** result

## Error Cases

| Condition | Behavior |
|-----------|----------|

## Dependencies

### Consumes

| Module | What is used |
|--------|-------------|

### Consumed By

| Module | What is used |
|--------|-------------|

## Change Log

| Date | Author | Change |
|------|--------|--------|
"#;
    fs::write(root.join("specs/dep/dep.spec.md"), spec).unwrap();

    specsync()
        .arg("check")
        .arg("--root")
        .arg(&root)
        .arg("--enforcement")
        .arg("strict")
        .assert()
        .failure()
        .stdout(predicate::str::contains("Dependency spec not found"));
}

#[test]
fn require_coverage_on_coverage_subcommand() {
    let tmp = TempDir::new().unwrap();
    let root = setup_minimal_project(&tmp);

    // Add uncovered file
    fs::write(root.join("src/auth/extra.ts"), "export function y() {}\n").unwrap();

    specsync()
        .arg("coverage")
        .arg("--require-coverage")
        .arg("100")
        .arg("--root")
        .arg(&root)
        .assert()
        .failure()
        .stdout(predicate::str::contains("--require-coverage"));
}

#[test]
fn generate_with_multiple_languages() {
    let tmp = TempDir::new().unwrap();
    let root = tmp.path().to_path_buf();

    write_config(&root, "specs", &["src"]);

    // Create modules with different languages, none with specs
    fs::create_dir_all(root.join("src/ts-svc")).unwrap();
    fs::write(
        root.join("src/ts-svc/index.ts"),
        "export function tsFunc() {}\n",
    )
    .unwrap();

    fs::create_dir_all(root.join("src/go-svc")).unwrap();
    fs::write(
        root.join("src/go-svc/main.go"),
        "package main\n\nfunc GoFunc() {}\n",
    )
    .unwrap();

    // Need at least one spec to avoid the "no spec files" early exit.
    // Create a dummy specced module.
    fs::create_dir_all(root.join("src/base")).unwrap();
    fs::write(root.join("src/base/base.ts"), "export function base() {}\n").unwrap();
    fs::create_dir_all(root.join("specs/base")).unwrap();
    let spec = valid_spec("base", &["src/base/base.ts"]);
    fs::write(root.join("specs/base/base.spec.md"), spec).unwrap();

    specsync()
        .arg("generate")
        .arg("--root")
        .arg(&root)
        .assert()
        .success()
        .stdout(predicate::str::contains("Generated"));

    // Both modules should have specs generated
    assert!(root.join("specs/ts-svc/ts-svc.spec.md").exists());
    assert!(root.join("specs/go-svc/go-svc.spec.md").exists());
}

#[test]
fn strict_on_coverage_subcommand() {
    let tmp = TempDir::new().unwrap();
    let root = tmp.path().to_path_buf();

    write_config(&root, "specs", &["src"]);

    fs::create_dir_all(root.join("src/warn")).unwrap();
    fs::write(
        root.join("src/warn/lib.ts"),
        "export function a() {}\nexport function b() {}\n",
    )
    .unwrap();

    fs::create_dir_all(root.join("specs/warn")).unwrap();
    // Only document one of two exports
    let spec = r#"---
module: warn
version: 1
status: active
files:
  - src/warn/lib.ts
db_tables: []
depends_on: []
---

# Warn

## Purpose

Something.

## Public API

### Exported Functions

| Function | Parameters | Returns | Description |
|----------|-----------|---------|-------------|
| `a` | none | void | Function a |

## Invariants

1. Ok.

## Behavioral Examples

### Scenario: Basic

- **Given** precondition
- **When** action
- **Then** result

## Error Cases

| Condition | Behavior |
|-----------|----------|

## Dependencies

### Consumes

| Module | What is used |
|--------|-------------|

### Consumed By

| Module | What is used |
|--------|-------------|

## Change Log

| Date | Author | Change |
|------|--------|--------|
"#;
    fs::write(root.join("specs/warn/warn.spec.md"), spec).unwrap();

    // --strict on coverage subcommand should also fail
    specsync()
        .arg("coverage")
        .arg("--strict")
        .arg("--root")
        .arg(&root)
        .assert()
        .failure()
        .stdout(predicate::str::contains("--strict mode"));
}

// ─── Actionable Error Messages Tests ─────────────────────────────────────

#[test]
fn check_shows_fix_suggestions() {
    let tmp = TempDir::new().unwrap();
    let root = tmp.path();

    fs::create_dir_all(root.join("src")).unwrap();
    fs::write(root.join("src/auth.ts"), "export function login() {}").unwrap();

    // Create a spec with a missing source file reference
    fs::create_dir_all(root.join("specs/auth")).unwrap();
    fs::write(
        root.join("specs/auth/auth.spec.md"),
        "---\nmodule: auth\nversion: 1\nstatus: active\nfiles:\n  - src/auht.ts\ndb_tables: []\ndepends_on: []\n---\n\n# Auth\n\n## Purpose\nAuth module\n\n## Public API\nNone\n\n## Invariants\n1. Valid\n\n## Behavioral Examples\n### Scenario: Basic\n- **Given** x\n- **When** y\n- **Then** z\n\n## Error Cases\n| Condition | Behavior |\n|-----------|----------|\n\n## Dependencies\nNone\n\n## Change Log\n| Date | Author | Change |\n|------|--------|--------|\n",
    )
    .unwrap();

    specsync()
        .args([
            "check",
            "--root",
            root.to_str().unwrap(),
            "--enforcement",
            "strict",
        ])
        .assert()
        .failure()
        .stdout(predicate::str::contains("Suggested fixes:"))
        .stdout(predicate::str::contains("Did you mean"));
}

// ─── YAML extraction integration tests ──────────────────────────────────

#[test]
fn check_yaml_source_file_extracts_top_level_keys() {
    let tmp = TempDir::new().unwrap();
    let root = tmp.path().to_path_buf();
    write_toml_config(&root, "");

    // Create a YAML source file
    fs::create_dir_all(root.join("src/ci")).unwrap();
    fs::write(
        root.join("src/ci/build.yml"),
        "name: CI\non: push\njobs:\n  test:\n    runs-on: ubuntu-latest\n  lint:\n    runs-on: ubuntu-latest\n",
    )
    .unwrap();

    // Create a spec that tracks the YAML file
    fs::create_dir_all(root.join("specs/ci")).unwrap();
    let spec = valid_spec("ci", &["src/ci/build.yml"]);
    fs::write(root.join("specs/ci/ci.spec.md"), spec).unwrap();

    // Check should pass (no missing source files)
    specsync()
        .args(["check", "--root"])
        .arg(&root)
        .assert()
        .success()
        .stdout(predicate::str::contains("0 failed"));
}

#[test]
fn check_yaml_with_document_separator_in_content() {
    let tmp = TempDir::new().unwrap();
    let root = tmp.path().to_path_buf();
    write_toml_config(&root, "");

    // Create a YAML file that uses document separators (common in Kubernetes manifests)
    fs::create_dir_all(root.join("src/k8s")).unwrap();
    fs::write(
        root.join("src/k8s/manifests.yml"),
        "apiVersion: apps/v1\nkind: Deployment\nmetadata:\n  name: my-app\n---\napiVersion: v1\nkind: Service\nmetadata:\n  name: my-svc\n",
    )
    .unwrap();

    fs::create_dir_all(root.join("specs/k8s")).unwrap();
    let spec = valid_spec("k8s", &["src/k8s/manifests.yml"]);
    fs::write(root.join("specs/k8s/k8s.spec.md"), spec).unwrap();

    // Check should still pass — the YAML extractor handles multi-doc files
    specsync()
        .args(["check", "--root"])
        .arg(&root)
        .assert()
        .success()
        .stdout(predicate::str::contains("0 failed"));
}

#[test]
fn check_yaml_with_anchors_and_nested_keys() {
    let tmp = TempDir::new().unwrap();
    let root = tmp.path().to_path_buf();
    write_toml_config(&root, "");

    fs::create_dir_all(root.join("src/docker")).unwrap();
    fs::write(
        root.join("src/docker/compose.yml"),
        "version: \"3.8\"\nservices:\n  web:\n    image: nginx\n  db:\n    image: postgres\nvolumes:\n  pgdata:\n    driver: local\n",
    )
    .unwrap();

    fs::create_dir_all(root.join("specs/docker")).unwrap();
    let spec = valid_spec("docker", &["src/docker/compose.yml"]);
    fs::write(root.join("specs/docker/docker.spec.md"), spec).unwrap();

    specsync()
        .args(["check", "--root"])
        .arg(&root)
        .assert()
        .success()
        .stdout(predicate::str::contains("0 failed"));
}

#[test]
fn check_github_actions_yaml_with_dotted_exports_passes_strict() {
    let tmp = TempDir::new().unwrap();
    let root = tmp.path().to_path_buf();
    write_toml_config(&root, "");

    fs::create_dir_all(root.join(".github/workflows")).unwrap();
    fs::write(
        root.join(".github/workflows/deploy.yml"),
        r#"name: Deploy Atlas
on:
  workflow_call:
    inputs:
      config:
        required: true
        type: string
      working-directory:
        required: false
        type: string
    outputs:
      atlas-enabled:
        value: ${{ jobs.deploy-atlas.outputs.enabled }}
permissions:
  contents: read
  id-token: write
jobs:
  deploy-atlas:
    runs-on: ubuntu-latest
"#,
    )
    .unwrap();

    fs::create_dir_all(root.join("specs/deploy")).unwrap();
    fs::write(
        root.join("specs/deploy/deploy.spec.md"),
        r#"---
module: deploy
version: 1
status: active
files:
  - .github/workflows/deploy.yml
db_tables: []
depends_on: []
---

# Deploy

## Purpose

Deploy Atlas through a reusable GitHub Actions workflow.

## Public API

### Exported YAML Symbols

| Symbol | Description |
|--------|-------------|
| `name` | Workflow name |
| `on` | Workflow trigger |
| `permissions` | Workflow permissions |
| `jobs` | Workflow jobs |
| `inputs.config` | Configuration input |
| `inputs.working-directory` | Working-directory input |
| `outputs.atlas-enabled` | Atlas output |
| `permissions.contents` | Contents permission |
| `permissions.id-token` | Identity-token permission |
| `jobs.deploy-atlas` | Deployment job |

## Invariants

1. Permissions remain least privilege.

## Behavioral Examples

### Scenario: Deploy

- **Given** a valid configuration
- **When** the reusable workflow runs
- **Then** Atlas is deployed

## Error Cases

| Condition | Behavior |
|-----------|----------|
| Invalid configuration | The workflow fails |

## Dependencies

None.

## Change Log

| Date | Change |
|------|--------|
| 2026-07-11 | Initial contract |
"#,
    )
    .unwrap();

    specsync()
        .args(["check", "--force", "--strict", "--root"])
        .arg(&root)
        .assert()
        .success()
        .stdout(predicate::str::contains("10/10 exports documented"))
        .stdout(predicate::str::contains("0 warning(s)"))
        .stdout(predicate::str::contains("0 failed"));
}

// ─── #245: Error messages include config file location ──────────────────

#[test]
fn error_messages_include_config_path() {
    let tmp = TempDir::new().unwrap();
    let root = tmp.path().to_path_buf();

    write_config(&root, "specs", &["src"]);
    fs::create_dir_all(root.join("specs/cfgmod")).unwrap();
    fs::create_dir_all(root.join("src/cfgmod")).unwrap();
    fs::write(root.join("src/cfgmod/index.ts"), "export function f() {}").unwrap();
    // Spec missing required sections — errors should mention config location
    let spec = r#"---
module: cfgmod
version: 1
status: active
files:
  - src/cfgmod/index.ts
---

## Purpose
Test module
"#;
    fs::write(root.join("specs/cfgmod/cfgmod.spec.md"), spec).unwrap();

    let output = specsync()
        .args(["check", "--root", root.to_str().unwrap(), "--force"])
        .output()
        .unwrap();
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        stdout.contains("check config:") || stdout.contains("specsync.json"),
        "Missing section errors should reference config file. Got:\n{stdout}"
    );
}

// ─── #249: File size warning uses configurable limit, no duplicate ──────

#[test]
fn file_size_warning_respects_max_spec_size_kb() {
    let tmp = TempDir::new().unwrap();
    let root = tmp.path().to_path_buf();

    let toml_config = r#"specs_dir = "specs"
source_dirs = ["src"]
required_sections = ["Purpose", "Public API", "Invariants", "Behavioral Examples", "Error Cases", "Dependencies", "Change Log"]
exclude_dirs = ["__tests__"]
exclude_patterns = ["**/__tests__/**"]

[rules]
max_spec_size_kb = 1
"#;
    fs::create_dir_all(root.join(".specsync")).unwrap();
    fs::write(root.join(".specsync/config.toml"), toml_config).unwrap();
    fs::create_dir_all(root.join("specs/bigmod")).unwrap();
    fs::create_dir_all(root.join("src/bigmod")).unwrap();
    fs::write(root.join("src/bigmod/index.ts"), "export function f() {}").unwrap();

    let mut big_spec = valid_spec("bigmod", &["src/bigmod/index.ts"]);
    while big_spec.len() < 2048 {
        big_spec.push_str("<!-- padding to exceed 1 KB -->\n");
    }
    fs::write(root.join("specs/bigmod/bigmod.spec.md"), &big_spec).unwrap();
    fs::write(
        root.join("specs/bigmod/requirements.md"),
        "# Requirements\n",
    )
    .unwrap();

    let output = specsync()
        .args(["check", "--root", root.to_str().unwrap(), "--force"])
        .output()
        .unwrap();
    let stdout = String::from_utf8_lossy(&output.stdout);

    assert!(
        stdout.contains("exceeds limit of 1 KB"),
        "Should warn at configured 1 KB limit. Got:\n{stdout}"
    );
    let count = stdout.matches("exceeds limit").count();
    assert_eq!(
        count, 1,
        "Should produce exactly one size warning, got {count}"
    );
}

// ─── specsync check: gate evaluation is not bypassed ─────────────────────

#[test]
fn check_no_specs_still_evaluates_requested_gate() {
    // Regression (H2): a project with source but no specs must still FAIL a
    // requested coverage/enforcement gate instead of short-circuiting to exit 0.
    let tmp = TempDir::new().unwrap();
    let root = tmp.path();
    write_config(root, "specs", &["src"]);
    fs::create_dir_all(root.join("src")).unwrap();
    fs::write(root.join("src/a.ts"), "export function a() {}\n").unwrap();

    // 0% coverage must fail a 100% requirement, and enforce-new must flag the
    // unspecced file.
    specsync()
        .current_dir(root)
        .args(["check", "--require-coverage", "100"])
        .assert()
        .failure();
    specsync()
        .current_dir(root)
        .args(["check", "--enforcement", "enforce-new"])
        .assert()
        .failure();
    // Default check (no gate requested) still succeeds informationally.
    specsync().current_dir(root).arg("check").assert().success();
}

#[test]
fn check_require_coverage_gate_fails_on_warm_cache() {
    // Regression (C2): the coverage gate must be evaluated even when the hash
    // cache is warm and no specs need re-validation — a warm run must not flip a
    // failing --require-coverage into exit 0.
    let tmp = TempDir::new().unwrap();
    let root = tmp.path();
    write_config(root, "specs", &["src"]);
    fs::create_dir_all(root.join("src")).unwrap();
    // 1 specced + 1 unspecced file → 50% coverage.
    fs::write(root.join("src/a.ts"), "export function a() {}\n").unwrap();
    fs::write(root.join("src/b.ts"), "export function b() {}\n").unwrap();
    fs::create_dir_all(root.join("specs/a")).unwrap();
    fs::write(
        root.join("specs/a/a.spec.md"),
        valid_spec("a", &["src/a.ts"]),
    )
    .unwrap();

    // Cold run populates the cache and fails the 90% gate.
    specsync()
        .current_dir(root)
        .args(["check", "--require-coverage", "90"])
        .assert()
        .failure();
    // Warm run (specs unchanged, served from cache) must ALSO fail.
    specsync()
        .current_dir(root)
        .args(["check", "--require-coverage", "90"])
        .assert()
        .failure();
}
