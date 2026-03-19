use assert_cmd::Command;
use predicates::prelude::*;
use std::fs;
use tempfile::TempDir;

// ─── Helpers ─────────────────────────────────────────────────────────────

/// Create a specsync binary command.
fn specsync() -> Command {
    Command::cargo_bin("specsync").unwrap()
}

/// A valid spec file that passes all checks.
fn valid_spec(module: &str, files: &[&str]) -> String {
    let files_yaml: String = files.iter().map(|f| format!("  - {f}\n")).collect();
    format!(
        r#"---
module: {module}
version: 1
status: active
files:
{files_yaml}db_tables: []
depends_on: []
---

# {title}

## Purpose

This module does something.

## Public API

### Exported Functions

| Function | Parameters | Returns | Description |
|----------|-----------|---------|-------------|

### Exported Types

| Type | Description |
|------|-------------|

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
"#,
        title = module
            .split('-')
            .map(|w| {
                let mut c = w.chars();
                match c.next() {
                    Some(ch) => ch.to_uppercase().to_string() + c.as_str(),
                    None => String::new(),
                }
            })
            .collect::<Vec<_>>()
            .join(" ")
    )
}

/// Write a specsync.json config to the given root.
fn write_config(root: &std::path::Path, specs_dir: &str, source_dirs: &[&str]) {
    let dirs: Vec<String> = source_dirs.iter().map(|d| format!("\"{d}\"")).collect();
    let config = format!(
        r#"{{
  "specsDir": "{specs_dir}",
  "sourceDirs": [{source_dirs}],
  "requiredSections": [
    "Purpose",
    "Public API",
    "Invariants",
    "Behavioral Examples",
    "Error Cases",
    "Dependencies",
    "Change Log"
  ],
  "excludeDirs": ["__tests__"],
  "excludePatterns": ["**/__tests__/**", "**/*.test.ts", "**/*.spec.ts"]
}}"#,
        source_dirs = dirs.join(", ")
    );
    fs::write(root.join("specsync.json"), config).unwrap();
}

/// Create a minimal project: config + specs dir + source dir + one spec + one source file.
fn setup_minimal_project(tmp: &TempDir) -> std::path::PathBuf {
    let root = tmp.path().to_path_buf();

    write_config(&root, "specs", &["src"]);

    // Source
    fs::create_dir_all(root.join("src/auth")).unwrap();
    fs::write(
        root.join("src/auth/service.ts"),
        "export function login() {}\nexport function logout() {}\n",
    )
    .unwrap();

    // Spec
    fs::create_dir_all(root.join("specs/auth")).unwrap();
    let spec = valid_spec("auth", &["src/auth/service.ts"]);
    fs::write(root.join("specs/auth/auth.spec.md"), spec).unwrap();

    root
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
        .assert()
        .failure()
        .stdout(predicate::str::contains("Source file not found"));
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
            "Export 'undocumented' not in spec",
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
        .assert()
        .failure()
        .stdout(predicate::str::contains(
            "Spec documents 'phantomExport' but no matching export found",
        ));
}

// ─── 2. specsync coverage ───────────────────────────────────────────────

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
        .stdout(predicate::str::contains("Created specsync.json"));

    let config_path = root.join("specsync.json");
    assert!(config_path.exists());
    let content = fs::read_to_string(&config_path).unwrap();
    assert!(content.contains("specsDir"));
    assert!(content.contains("sourceDirs"));
    assert!(content.contains("requiredSections"));
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

// ─── 8. Multi-language ──────────────────────────────────────────────────

#[test]
fn multi_lang_typescript() {
    let tmp = TempDir::new().unwrap();
    let root = tmp.path().to_path_buf();

    write_config(&root, "specs", &["src"]);

    fs::create_dir_all(root.join("src/ts-mod")).unwrap();
    fs::write(
        root.join("src/ts-mod/index.ts"),
        "export function greet(name: string): string { return `Hi ${name}`; }\nexport type Greeting = string;\n",
    )
    .unwrap();

    fs::create_dir_all(root.join("specs/ts-mod")).unwrap();
    let spec = valid_spec("ts-mod", &["src/ts-mod/index.ts"]);
    fs::write(root.join("specs/ts-mod/ts-mod.spec.md"), spec).unwrap();

    specsync()
        .arg("check")
        .arg("--root")
        .arg(&root)
        .assert()
        .success()
        .stdout(predicate::str::contains("specs checked"));
}

#[test]
fn multi_lang_rust() {
    let tmp = TempDir::new().unwrap();
    let root = tmp.path().to_path_buf();

    write_config(&root, "specs", &["src"]);

    fs::create_dir_all(root.join("src/rs-mod")).unwrap();
    fs::write(
        root.join("src/rs-mod/lib.rs"),
        "pub fn add(a: i32, b: i32) -> i32 { a + b }\npub struct Config { pub name: String }\n",
    )
    .unwrap();

    fs::create_dir_all(root.join("specs/rs-mod")).unwrap();
    let spec = valid_spec("rs-mod", &["src/rs-mod/lib.rs"]);
    fs::write(root.join("specs/rs-mod/rs-mod.spec.md"), spec).unwrap();

    specsync()
        .arg("check")
        .arg("--root")
        .arg(&root)
        .assert()
        .success();
}

#[test]
fn multi_lang_go() {
    let tmp = TempDir::new().unwrap();
    let root = tmp.path().to_path_buf();

    write_config(&root, "specs", &["src"]);

    fs::create_dir_all(root.join("src/gomod")).unwrap();
    fs::write(
        root.join("src/gomod/handler.go"),
        "package gomod\n\nfunc HandleRequest() error { return nil }\n\ntype Request struct {\n\tBody string\n}\n",
    )
    .unwrap();

    fs::create_dir_all(root.join("specs/gomod")).unwrap();
    let spec = valid_spec("gomod", &["src/gomod/handler.go"]);
    fs::write(root.join("specs/gomod/gomod.spec.md"), spec).unwrap();

    specsync()
        .arg("check")
        .arg("--root")
        .arg(&root)
        .assert()
        .success();
}

#[test]
fn multi_lang_python() {
    let tmp = TempDir::new().unwrap();
    let root = tmp.path().to_path_buf();

    write_config(&root, "specs", &["src"]);

    fs::create_dir_all(root.join("src/pymod")).unwrap();
    fs::write(
        root.join("src/pymod/core.py"),
        "def process_data(data):\n    return data\n\nclass DataProcessor:\n    pass\n",
    )
    .unwrap();

    fs::create_dir_all(root.join("specs/pymod")).unwrap();
    let spec = valid_spec("pymod", &["src/pymod/core.py"]);
    fs::write(root.join("specs/pymod/pymod.spec.md"), spec).unwrap();

    specsync()
        .arg("check")
        .arg("--root")
        .arg(&root)
        .assert()
        .success();
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

// ─── Provider / Multi-Agent Tests ────────────────────────────────────────

#[test]
fn provider_flag_unknown_provider_errors() {
    let tmp = TempDir::new().unwrap();
    let root = setup_minimal_project(&tmp);

    specsync()
        .arg("generate")
        .arg("--ai")
        .arg("--provider")
        .arg("nonexistent")
        .arg("--root")
        .arg(&root)
        .assert()
        .failure()
        .stderr(predicate::str::contains("Unknown provider"));
}

#[test]
fn provider_flag_implies_ai() {
    // --provider without --ai should still enable AI mode (and fail if binary not found)
    let tmp = TempDir::new().unwrap();
    let root = setup_minimal_project(&tmp);

    specsync()
        .arg("generate")
        .arg("--provider")
        .arg("cursor")
        .arg("--root")
        .arg(&root)
        .assert()
        .failure()
        .stderr(predicate::str::contains("cursor"));
}

#[test]
fn ai_provider_config_field_is_respected() {
    let tmp = TempDir::new().unwrap();
    let root = setup_minimal_project(&tmp);

    // Overwrite config with aiProvider set to cursor (not installed)
    let config = serde_json::json!({
        "specsDir": "specs",
        "sourceDirs": ["src"],
        "aiProvider": "cursor",
        "requiredSections": ["Purpose", "Public API", "Invariants", "Behavioral Examples", "Error Cases", "Dependencies", "Change Log"],
        "excludeDirs": ["__tests__"],
        "excludePatterns": ["**/__tests__/**"]
    });
    fs::write(
        root.join("specsync.json"),
        serde_json::to_string_pretty(&config).unwrap(),
    )
    .unwrap();

    specsync()
        .arg("generate")
        .arg("--ai")
        .arg("--root")
        .arg(&root)
        .assert()
        .failure()
        .stderr(predicate::str::contains("cursor"));
}

#[test]
fn ai_command_overrides_ai_provider() {
    let tmp = TempDir::new().unwrap();
    let root = setup_minimal_project(&tmp);

    // Config has both aiProvider and aiCommand — aiCommand ("false") wins over aiProvider
    // The "false" command exits 1, AI falls back to template, but stderr shows it tried
    let config = serde_json::json!({
        "specsDir": "specs",
        "sourceDirs": ["src"],
        "aiProvider": "claude",
        "aiCommand": "false",
        "requiredSections": ["Purpose", "Public API", "Invariants", "Behavioral Examples", "Error Cases", "Dependencies", "Change Log"],
        "excludeDirs": ["__tests__"],
        "excludePatterns": ["**/__tests__/**"]
    });
    fs::write(
        root.join("specsync.json"),
        serde_json::to_string_pretty(&config).unwrap(),
    )
    .unwrap();

    // Create unspecced module so generation is triggered
    fs::create_dir_all(root.join("src/newmod")).unwrap();
    fs::write(root.join("src/newmod/lib.rs"), "pub fn hello() {}").unwrap();

    // The command succeeds (falls back to template) but stderr shows AI was attempted & failed
    specsync()
        .arg("generate")
        .arg("--ai")
        .arg("--root")
        .arg(&root)
        .assert()
        .success()
        .stderr(predicate::str::contains("AI generation failed"));
}

#[test]
fn cli_provider_overrides_config_provider() {
    let tmp = TempDir::new().unwrap();
    let root = setup_minimal_project(&tmp);

    // Config says claude, CLI says cursor — cursor should win
    let config = serde_json::json!({
        "specsDir": "specs",
        "sourceDirs": ["src"],
        "aiProvider": "claude",
        "requiredSections": ["Purpose", "Public API", "Invariants", "Behavioral Examples", "Error Cases", "Dependencies", "Change Log"],
        "excludeDirs": ["__tests__"],
        "excludePatterns": ["**/__tests__/**"]
    });
    fs::write(
        root.join("specsync.json"),
        serde_json::to_string_pretty(&config).unwrap(),
    )
    .unwrap();

    specsync()
        .arg("generate")
        .arg("--provider")
        .arg("cursor")
        .arg("--root")
        .arg(&root)
        .assert()
        .failure()
        .stderr(predicate::str::contains("cursor"));
}

#[test]
fn ai_model_config_used_with_ollama_provider() {
    let tmp = TempDir::new().unwrap();
    let root = setup_minimal_project(&tmp);

    // aiProvider=ollama with custom model — should mention ollama in error
    let config = serde_json::json!({
        "specsDir": "specs",
        "sourceDirs": ["src"],
        "aiProvider": "ollama",
        "aiModel": "codellama",
        "requiredSections": ["Purpose", "Public API", "Invariants", "Behavioral Examples", "Error Cases", "Dependencies", "Change Log"],
        "excludeDirs": ["__tests__"],
        "excludePatterns": ["**/__tests__/**"]
    });
    fs::write(
        root.join("specsync.json"),
        serde_json::to_string_pretty(&config).unwrap(),
    )
    .unwrap();

    specsync()
        .arg("generate")
        .arg("--ai")
        .arg("--root")
        .arg(&root)
        .assert()
        .failure()
        .stderr(predicate::str::contains("ollama"));
}

// ─── 8. Direct API provider tests ───────────────────────────────────────

#[test]
fn anthropic_provider_requires_api_key() {
    let tmp = TempDir::new().unwrap();
    let root = setup_minimal_project(&tmp);

    // aiProvider=anthropic without ANTHROPIC_API_KEY should error
    let config = serde_json::json!({
        "specsDir": "specs",
        "sourceDirs": ["src"],
        "aiProvider": "anthropic",
        "requiredSections": ["Purpose", "Public API", "Invariants", "Behavioral Examples", "Error Cases", "Dependencies", "Change Log"],
        "excludeDirs": ["__tests__"],
        "excludePatterns": ["**/__tests__/**"]
    });
    fs::write(
        root.join("specsync.json"),
        serde_json::to_string_pretty(&config).unwrap(),
    )
    .unwrap();

    specsync()
        .arg("generate")
        .arg("--ai")
        .arg("--root")
        .arg(&root)
        .env_remove("ANTHROPIC_API_KEY")
        .assert()
        .failure()
        .stderr(predicate::str::contains("ANTHROPIC_API_KEY"));
}

#[test]
fn openai_provider_requires_api_key() {
    let tmp = TempDir::new().unwrap();
    let root = setup_minimal_project(&tmp);

    // aiProvider=openai without OPENAI_API_KEY should error
    let config = serde_json::json!({
        "specsDir": "specs",
        "sourceDirs": ["src"],
        "aiProvider": "openai",
        "requiredSections": ["Purpose", "Public API", "Invariants", "Behavioral Examples", "Error Cases", "Dependencies", "Change Log"],
        "excludeDirs": ["__tests__"],
        "excludePatterns": ["**/__tests__/**"]
    });
    fs::write(
        root.join("specsync.json"),
        serde_json::to_string_pretty(&config).unwrap(),
    )
    .unwrap();

    specsync()
        .arg("generate")
        .arg("--ai")
        .arg("--root")
        .arg(&root)
        .env_remove("OPENAI_API_KEY")
        .assert()
        .failure()
        .stderr(predicate::str::contains("OPENAI_API_KEY"));
}

#[test]
fn provider_flag_anthropic_requires_api_key() {
    let tmp = TempDir::new().unwrap();
    let root = setup_minimal_project(&tmp);

    specsync()
        .arg("generate")
        .arg("--provider")
        .arg("anthropic")
        .arg("--root")
        .arg(&root)
        .env_remove("ANTHROPIC_API_KEY")
        .assert()
        .failure()
        .stderr(predicate::str::contains("ANTHROPIC_API_KEY"));
}

#[test]
fn provider_flag_openai_requires_api_key() {
    let tmp = TempDir::new().unwrap();
    let root = setup_minimal_project(&tmp);

    specsync()
        .arg("generate")
        .arg("--provider")
        .arg("openai")
        .arg("--root")
        .arg(&root)
        .env_remove("OPENAI_API_KEY")
        .assert()
        .failure()
        .stderr(predicate::str::contains("OPENAI_API_KEY"));
}

#[test]
fn anthropic_api_alias_works() {
    // "anthropic-api" should be accepted as an alias for "anthropic"
    let tmp = TempDir::new().unwrap();
    let root = setup_minimal_project(&tmp);

    specsync()
        .arg("generate")
        .arg("--provider")
        .arg("anthropic-api")
        .arg("--root")
        .arg(&root)
        .env_remove("ANTHROPIC_API_KEY")
        .assert()
        .failure()
        .stderr(predicate::str::contains("ANTHROPIC_API_KEY"));
}

#[test]
fn openai_api_alias_works() {
    // "openai-api" should be accepted as an alias for "openai"
    let tmp = TempDir::new().unwrap();
    let root = setup_minimal_project(&tmp);

    specsync()
        .arg("generate")
        .arg("--provider")
        .arg("openai-api")
        .arg("--root")
        .arg(&root)
        .env_remove("OPENAI_API_KEY")
        .assert()
        .failure()
        .stderr(predicate::str::contains("OPENAI_API_KEY"));
}

#[test]
fn ai_api_key_config_field_used_for_anthropic() {
    let tmp = TempDir::new().unwrap();
    let root = setup_minimal_project(&tmp);

    // Create unspecced module so generation is triggered
    fs::create_dir_all(root.join("src/newmod")).unwrap();
    fs::write(root.join("src/newmod/lib.rs"), "pub fn hello() {}").unwrap();

    // Set aiProvider=anthropic with aiApiKey in config (fake key)
    // This should attempt the API call (and fail with auth error), proving
    // the key was picked up from config rather than erroring about missing key
    let config = serde_json::json!({
        "specsDir": "specs",
        "sourceDirs": ["src"],
        "aiProvider": "anthropic",
        "aiApiKey": "sk-ant-test-fake-key",
        "requiredSections": ["Purpose", "Public API", "Invariants", "Behavioral Examples", "Error Cases", "Dependencies", "Change Log"],
        "excludeDirs": ["__tests__"],
        "excludePatterns": ["**/__tests__/**"]
    });
    fs::write(
        root.join("specsync.json"),
        serde_json::to_string_pretty(&config).unwrap(),
    )
    .unwrap();

    // Should succeed (API call fails, falls back to template)
    specsync()
        .arg("generate")
        .arg("--ai")
        .arg("--root")
        .arg(&root)
        .env_remove("ANTHROPIC_API_KEY")
        .assert()
        .success()
        .stderr(predicate::str::contains("AI generation failed"));
}

#[test]
fn unknown_provider_lists_api_options() {
    // Error message for unknown provider should mention anthropic and openai
    let tmp = TempDir::new().unwrap();
    let root = setup_minimal_project(&tmp);

    specsync()
        .arg("generate")
        .arg("--provider")
        .arg("bogus")
        .arg("--root")
        .arg(&root)
        .assert()
        .failure()
        .stderr(predicate::str::contains("anthropic").and(predicate::str::contains("openai")));
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

    let config: serde_json::Value =
        serde_json::from_str(&fs::read_to_string(root.join("specsync.json")).unwrap()).unwrap();
    let dirs = config["sourceDirs"].as_array().unwrap();
    assert_eq!(dirs.len(), 1);
    assert_eq!(dirs[0], "src");
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

    let config: serde_json::Value =
        serde_json::from_str(&fs::read_to_string(root.join("specsync.json")).unwrap()).unwrap();
    let dirs = config["sourceDirs"].as_array().unwrap();
    assert_eq!(dirs.len(), 1);
    assert_eq!(dirs[0], "lib");
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

    let config: serde_json::Value =
        serde_json::from_str(&fs::read_to_string(root.join("specsync.json")).unwrap()).unwrap();
    let dirs = config["sourceDirs"].as_array().unwrap();
    assert_eq!(dirs.len(), 2);
    assert_eq!(dirs[0], "lib");
    assert_eq!(dirs[1], "src");
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

    let config: serde_json::Value =
        serde_json::from_str(&fs::read_to_string(root.join("specsync.json")).unwrap()).unwrap();
    let dirs = config["sourceDirs"].as_array().unwrap();
    assert_eq!(dirs.len(), 1);
    assert_eq!(dirs[0], "app");
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

    let config: serde_json::Value =
        serde_json::from_str(&fs::read_to_string(root.join("specsync.json")).unwrap()).unwrap();
    let dirs = config["sourceDirs"].as_array().unwrap();
    assert_eq!(dirs.len(), 1);
    assert_eq!(dirs[0], "src");
}

// ─── MCP Server Tests ──────────────────────────────────────────────────────

/// Send JSON-RPC requests to the MCP server via stdin and capture stdout.
fn mcp_request(root: &std::path::Path, requests: &[serde_json::Value]) -> Vec<serde_json::Value> {
    let input: String = requests
        .iter()
        .map(|r| serde_json::to_string(r).unwrap())
        .collect::<Vec<_>>()
        .join("\n")
        + "\n";

    let output = specsync()
        .arg("mcp")
        .arg("--root")
        .arg(root)
        .write_stdin(input)
        .output()
        .expect("failed to run mcp");

    let stdout = String::from_utf8_lossy(&output.stdout);
    stdout
        .lines()
        .filter(|l| !l.trim().is_empty())
        .map(|l| serde_json::from_str(l).expect("invalid JSON-RPC response"))
        .collect()
}

#[test]
fn mcp_initialize_returns_capabilities() {
    let tmp = TempDir::new().unwrap();
    let root = tmp.path();

    let responses = mcp_request(
        root,
        &[serde_json::json!({
            "jsonrpc": "2.0",
            "id": 1,
            "method": "initialize",
            "params": {}
        })],
    );

    assert_eq!(responses.len(), 1);
    let result = &responses[0]["result"];
    assert_eq!(result["serverInfo"]["name"], "specsync");
    assert!(result["capabilities"]["tools"].is_object());
}

#[test]
fn mcp_tools_list_returns_all_tools() {
    let tmp = TempDir::new().unwrap();
    let root = tmp.path();

    let responses = mcp_request(
        root,
        &[serde_json::json!({
            "jsonrpc": "2.0",
            "id": 1,
            "method": "tools/list",
            "params": {}
        })],
    );

    let tools = responses[0]["result"]["tools"].as_array().unwrap();
    let tool_names: Vec<&str> = tools
        .iter()
        .map(|t| t["name"].as_str().unwrap())
        .collect();
    assert!(tool_names.contains(&"specsync_check"));
    assert!(tool_names.contains(&"specsync_coverage"));
    assert!(tool_names.contains(&"specsync_generate"));
    assert!(tool_names.contains(&"specsync_list_specs"));
    assert!(tool_names.contains(&"specsync_init"));
}

#[test]
fn mcp_tool_check_validates_specs() {
    let tmp = TempDir::new().unwrap();
    let root = setup_minimal_project(&tmp);

    let responses = mcp_request(
        &root,
        &[serde_json::json!({
            "jsonrpc": "2.0",
            "id": 1,
            "method": "tools/call",
            "params": {
                "name": "specsync_check",
                "arguments": {}
            }
        })],
    );

    let content = &responses[0]["result"]["content"][0]["text"];
    let result: serde_json::Value =
        serde_json::from_str(content.as_str().unwrap()).unwrap();
    assert!(result["passed"].as_bool().unwrap());
    assert_eq!(result["specs_checked"].as_u64().unwrap(), 1);
}

#[test]
fn mcp_tool_coverage_returns_metrics() {
    let tmp = TempDir::new().unwrap();
    let root = setup_minimal_project(&tmp);

    let responses = mcp_request(
        &root,
        &[serde_json::json!({
            "jsonrpc": "2.0",
            "id": 1,
            "method": "tools/call",
            "params": {
                "name": "specsync_coverage",
                "arguments": {}
            }
        })],
    );

    let content = &responses[0]["result"]["content"][0]["text"];
    let result: serde_json::Value =
        serde_json::from_str(content.as_str().unwrap()).unwrap();
    assert!(result["files_total"].as_u64().unwrap() > 0);
    assert!(result["file_coverage"].is_number());
}

#[test]
fn mcp_tool_init_creates_config() {
    let tmp = TempDir::new().unwrap();
    let root = tmp.path();
    fs::create_dir_all(root.join("src")).unwrap();
    fs::write(root.join("src/main.rs"), "fn main() {}").unwrap();

    let responses = mcp_request(
        root,
        &[serde_json::json!({
            "jsonrpc": "2.0",
            "id": 1,
            "method": "tools/call",
            "params": {
                "name": "specsync_init",
                "arguments": {}
            }
        })],
    );

    let content = &responses[0]["result"]["content"][0]["text"];
    let result: serde_json::Value =
        serde_json::from_str(content.as_str().unwrap()).unwrap();
    assert!(result["created"].as_bool().unwrap());
    assert!(root.join("specsync.json").exists());
}

#[test]
fn mcp_tool_list_specs_returns_spec_info() {
    let tmp = TempDir::new().unwrap();
    let root = setup_minimal_project(&tmp);

    let responses = mcp_request(
        &root,
        &[serde_json::json!({
            "jsonrpc": "2.0",
            "id": 1,
            "method": "tools/call",
            "params": {
                "name": "specsync_list_specs",
                "arguments": {}
            }
        })],
    );

    let content = &responses[0]["result"]["content"][0]["text"];
    let result: serde_json::Value =
        serde_json::from_str(content.as_str().unwrap()).unwrap();
    assert!(result["count"].as_u64().unwrap() >= 1);
    let specs = result["specs"].as_array().unwrap();
    assert!(specs[0]["module"].is_string());
    assert!(specs[0]["path"].is_string());
}

#[test]
fn mcp_unknown_tool_returns_error() {
    let tmp = TempDir::new().unwrap();
    let root = tmp.path();

    let responses = mcp_request(
        root,
        &[serde_json::json!({
            "jsonrpc": "2.0",
            "id": 1,
            "method": "tools/call",
            "params": {
                "name": "nonexistent_tool",
                "arguments": {}
            }
        })],
    );

    let result = &responses[0]["result"];
    assert!(result["isError"].as_bool().unwrap());
}

#[test]
fn mcp_ping_returns_empty_result() {
    let tmp = TempDir::new().unwrap();
    let root = tmp.path();

    let responses = mcp_request(
        root,
        &[serde_json::json!({
            "jsonrpc": "2.0",
            "id": 1,
            "method": "ping"
        })],
    );

    assert_eq!(responses.len(), 1);
    assert!(responses[0]["result"].is_object());
}
