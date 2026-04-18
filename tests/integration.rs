use assert_cmd::Command;
use predicates::prelude::*;
use std::fs;
use tempfile::TempDir;

// ─── Helpers ─────────────────────────────────────────────────────────────

/// Create a specsync binary command.
/// Clears GitHub Actions env vars so PR-base auto-detection doesn't interfere with tests.
fn specsync() -> Command {
    let mut cmd = Command::cargo_bin("specsync").unwrap();
    cmd.env_remove("GITHUB_EVENT_NAME");
    cmd.env_remove("GITHUB_BASE_REF");
    cmd
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
        .arg("--enforcement")
        .arg("strict")
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

#[test]
fn multi_lang_php() {
    let tmp = TempDir::new().unwrap();
    let root = tmp.path().to_path_buf();

    write_config(&root, "specs", &["src"]);

    fs::create_dir_all(root.join("src/phpmod")).unwrap();
    fs::write(
        root.join("src/phpmod/Service.php"),
        r#"<?php

namespace App\Auth;

class AuthService {
    public const DEFAULT_TTL = 3600;

    public function validate(string $token): bool {
        return true;
    }

    private function internalCheck(): void {}
}

interface Authenticator {
    public function authenticate(): bool;
}

function standalone_helper(): void {}
"#,
    )
    .unwrap();

    fs::create_dir_all(root.join("specs/phpmod")).unwrap();
    let spec = valid_spec("phpmod", &["src/phpmod/Service.php"]);
    fs::write(root.join("specs/phpmod/phpmod.spec.md"), spec).unwrap();

    specsync()
        .arg("check")
        .arg("--root")
        .arg(&root)
        .assert()
        .success()
        .stdout(predicate::str::contains("specs checked"));
}

#[test]
fn multi_lang_ruby() {
    let tmp = TempDir::new().unwrap();
    let root = tmp.path().to_path_buf();

    write_config(&root, "specs", &["src"]);

    fs::create_dir_all(root.join("src/rbmod")).unwrap();
    fs::write(
        root.join("src/rbmod/service.rb"),
        r#"
module Authentication
  class AuthService
    DEFAULT_TTL = 3600

    attr_reader :token

    def validate(token)
      true
    end

    def self.create(config)
      new
    end

    private

    def internal_check
      false
    end
  end
end

def standalone_helper
  true
end
"#,
    )
    .unwrap();

    fs::create_dir_all(root.join("specs/rbmod")).unwrap();
    let spec = valid_spec("rbmod", &["src/rbmod/service.rb"]);
    fs::write(root.join("specs/rbmod/rbmod.spec.md"), spec).unwrap();

    specsync()
        .arg("check")
        .arg("--root")
        .arg(&root)
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

// ─── Provider / Multi-Agent Tests ────────────────────────────────────────

#[test]
fn provider_flag_unknown_provider_errors() {
    let tmp = TempDir::new().unwrap();
    let root = setup_minimal_project(&tmp);

    specsync()
        .arg("generate")
        .arg("--provider")
        .arg("nonexistent")
        .arg("--root")
        .arg(&root)
        .assert()
        .failure()
        .stderr(predicate::str::contains("Unknown provider"));
}

#[test]
fn provider_flag_enables_ai() {
    // --provider enables AI mode (and fails if binary not found)
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
        .stderr(predicate::str::contains("cursor").or(predicate::str::contains("Cursor")));
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
        .arg("--provider")
        .arg("auto")
        .arg("--root")
        .arg(&root)
        .assert()
        .failure()
        .stderr(predicate::str::contains("cursor").or(predicate::str::contains("Cursor")));
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
        .arg("--provider")
        .arg("auto")
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
        .stderr(predicate::str::contains("cursor").or(predicate::str::contains("Cursor")));
}

#[test]
fn ai_model_config_used_with_ollama_provider() {
    let tmp = TempDir::new().unwrap();
    let root = setup_minimal_project(&tmp);

    // Add an uncovered source file so `generate` actually attempts AI generation
    fs::create_dir_all(root.join("src/billing")).unwrap();
    fs::write(
        root.join("src/billing/invoice.ts"),
        "export function createInvoice() {}\n",
    )
    .unwrap();

    // aiProvider=ollama with custom model — should mention ollama in error.
    // Use an empty PATH so ollama binary is not found, even if installed locally.
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
        .arg("--provider")
        .arg("auto")
        .env("PATH", "")
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
        .arg("--provider")
        .arg("auto")
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
        .arg("--provider")
        .arg("auto")
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
        .arg("--provider")
        .arg("auto")
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
    let tool_names: Vec<&str> = tools.iter().map(|t| t["name"].as_str().unwrap()).collect();
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
    let result: serde_json::Value = serde_json::from_str(content.as_str().unwrap()).unwrap();
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
    let result: serde_json::Value = serde_json::from_str(content.as_str().unwrap()).unwrap();
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
    let result: serde_json::Value = serde_json::from_str(content.as_str().unwrap()).unwrap();
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
    let result: serde_json::Value = serde_json::from_str(content.as_str().unwrap()).unwrap();
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

// ─── TOML Config Tests ──────────────────────────────────────────────────

#[test]
fn toml_config_is_loaded() {
    let tmp = TempDir::new().unwrap();
    let root = tmp.path();

    fs::create_dir_all(root.join("lib")).unwrap();
    fs::write(root.join("lib/utils.ts"), "export function helper() {}").unwrap();

    // Write .specsync.toml instead of specsync.json
    fs::write(
        root.join(".specsync.toml"),
        r#"
specs_dir = "specs"
source_dirs = ["lib"]
required_sections = ["Purpose", "Public API"]
"#,
    )
    .unwrap();

    fs::create_dir_all(root.join("specs/utils")).unwrap();
    fs::write(
        root.join("specs/utils/utils.spec.md"),
        "---\nmodule: utils\nversion: 1\nstatus: active\nfiles:\n  - lib/utils.ts\ndb_tables: []\ndepends_on: []\n---\n\n# Utils\n\n## Purpose\n\nHelper utilities.\n\n## Public API\n\n| Function | Description |\n|----------|-------------|\n| `helper` | Helps |\n",
    )
    .unwrap();

    specsync()
        .args(["check", "--root", root.to_str().unwrap()])
        .assert()
        .success()
        .stdout(predicate::str::contains("1 specs checked"));
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

// ─── MCP Score Tool Tests ────────────────────────────────────────────────

#[test]
fn mcp_score_tool_returns_grades() {
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

    let responses = mcp_request(
        root,
        &[
            serde_json::json!({
                "jsonrpc": "2.0",
                "id": 1,
                "method": "initialize",
                "params": { "capabilities": {} }
            }),
            serde_json::json!({
                "jsonrpc": "2.0",
                "id": 2,
                "method": "tools/call",
                "params": {
                    "name": "specsync_score",
                    "arguments": {}
                }
            }),
        ],
    );

    let score_result = &responses[1]["result"]["content"][0]["text"];
    let score_json: serde_json::Value =
        serde_json::from_str(score_result.as_str().unwrap()).unwrap();
    assert!(score_json["average_score"].is_number());
    assert!(score_json["grade"].is_string());
}

// ─── Fix Flag Tests ─────────────────────────────────────────────────────

#[test]
fn fix_adds_undocumented_exports_to_spec() {
    let tmp = TempDir::new().unwrap();
    let root = tmp.path();

    write_config(root, "specs", &["src"]);

    // Source file with two exports
    fs::create_dir_all(root.join("src/auth")).unwrap();
    fs::write(
        root.join("src/auth/service.ts"),
        "export function login() {}\nexport function logout() {}\nexport const TOKEN_TTL = 3600;\n",
    )
    .unwrap();

    // Spec that documents NONE of the exports (empty Public API table)
    fs::create_dir_all(root.join("specs/auth")).unwrap();
    fs::write(
        root.join("specs/auth/auth.spec.md"),
        valid_spec("auth", &["src/auth/service.ts"]),
    )
    .unwrap();

    // Run check --fix
    specsync()
        .args(["check", "--fix", "--root", root.to_str().unwrap()])
        .assert()
        .success();

    // Verify the spec was modified to include the exports
    let updated = fs::read_to_string(root.join("specs/auth/auth.spec.md")).unwrap();
    assert!(
        updated.contains("`login`"),
        "Expected spec to contain `login` after --fix"
    );
    assert!(
        updated.contains("`logout`"),
        "Expected spec to contain `logout` after --fix"
    );
    assert!(
        updated.contains("`TOKEN_TTL`"),
        "Expected spec to contain `TOKEN_TTL` after --fix"
    );
    assert!(
        updated.contains("<!-- TODO: describe -->"),
        "Expected stub descriptions"
    );
}

#[test]
fn fix_does_not_duplicate_already_documented_exports() {
    let tmp = TempDir::new().unwrap();
    let root = tmp.path();

    write_config(root, "specs", &["src"]);

    fs::create_dir_all(root.join("src/auth")).unwrap();
    fs::write(
        root.join("src/auth/service.ts"),
        "export function login() {}\nexport function logout() {}\n",
    )
    .unwrap();

    // Spec already documents login but not logout
    fs::create_dir_all(root.join("specs/auth")).unwrap();
    let spec_with_login = r#"---
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
| `login` | Authenticates a user |

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
    fs::write(root.join("specs/auth/auth.spec.md"), spec_with_login).unwrap();

    // Run --fix
    specsync()
        .args(["check", "--fix", "--root", root.to_str().unwrap()])
        .assert()
        .success();

    let updated = fs::read_to_string(root.join("specs/auth/auth.spec.md")).unwrap();

    // login should appear exactly once (not duplicated)
    let login_count = updated.matches("`login`").count();
    assert_eq!(
        login_count, 1,
        "login should not be duplicated; found {login_count} times"
    );

    // logout should have been added
    assert!(
        updated.contains("`logout`"),
        "Expected spec to contain `logout` after --fix"
    );
}

#[test]
fn fix_creates_public_api_section_when_missing() {
    let tmp = TempDir::new().unwrap();
    let root = tmp.path();

    write_config(root, "specs", &["src"]);

    fs::create_dir_all(root.join("src/utils")).unwrap();
    fs::write(
        root.join("src/utils/helper.ts"),
        "export function doStuff() {}\n",
    )
    .unwrap();

    // Spec with no Public API section at all
    fs::create_dir_all(root.join("specs/utils")).unwrap();
    let spec_no_api = r#"---
module: utils
version: 1
status: active
files:
  - src/utils/helper.ts
db_tables: []
depends_on: []
---

# Utils

## Purpose

Utility functions.

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
    fs::write(root.join("specs/utils/utils.spec.md"), spec_no_api).unwrap();

    specsync()
        .args(["check", "--fix", "--root", root.to_str().unwrap()])
        .assert()
        .success();

    let updated = fs::read_to_string(root.join("specs/utils/utils.spec.md")).unwrap();
    assert!(
        updated.contains("## Public API"),
        "Expected --fix to create Public API section"
    );
    assert!(
        updated.contains("`doStuff`"),
        "Expected doStuff to be added"
    );
}

#[test]
fn fix_with_json_output() {
    let tmp = TempDir::new().unwrap();
    let root = tmp.path();

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

    // --fix with --json should still work and produce valid JSON
    let output = specsync()
        .args(["check", "--fix", "--json", "--root", root.to_str().unwrap()])
        .output()
        .unwrap();

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    // The auto_fix_specs function may print non-JSON lines before the JSON output,
    // so find the JSON object in the output
    let json_start = stdout.find('{').expect("Expected JSON in output");
    let json_str = &stdout[json_start..];
    let json: serde_json::Value = serde_json::from_str(json_str.trim()).unwrap();
    assert!(json["specs_checked"].is_number());
}

// Regression: --fix used to insert new rows at the end of ## Public API, which put
// them inside non-export subsections (e.g. ### API Endpoints). get_spec_symbols skips
// non-export subsections, so the symbol remained "undocumented" and --fix would append
// it again on every run, producing duplicates.
#[test]
fn fix_does_not_duplicate_when_non_export_subsections_present() {
    let tmp = TempDir::new().unwrap();
    let root = tmp.path();

    write_config(root, "specs", &["src"]);

    fs::create_dir_all(root.join("src/auth")).unwrap();
    fs::write(
        root.join("src/auth/service.ts"),
        "export function login() {}\nexport function logout() {}\n",
    )
    .unwrap();

    // Spec has login documented under a recognized export header, plus a
    // non-export ### API Endpoints subsection that would previously swallow new rows.
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

### Exported Functions

| Export | Description |
|--------|-------------|
| `login` | Authenticates a user |

### API Endpoints

| Endpoint | Method | Description |
|----------|--------|-------------|
| `/login` | POST | Login endpoint |

## Change Log

| Date | Author | Change |
|------|--------|--------|
"#;
    fs::write(root.join("specs/auth/auth.spec.md"), spec).unwrap();

    // First --fix run: should add logout
    specsync()
        .args(["check", "--fix", "--root", root.to_str().unwrap()])
        .assert()
        .success();

    let after_first = fs::read_to_string(root.join("specs/auth/auth.spec.md")).unwrap();
    assert!(
        after_first.contains("`logout`"),
        "logout should be added after first --fix"
    );

    // Second --fix run: logout must not be duplicated
    specsync()
        .args(["check", "--fix", "--root", root.to_str().unwrap()])
        .assert()
        .success();

    let after_second = fs::read_to_string(root.join("specs/auth/auth.spec.md")).unwrap();
    let logout_count = after_second.matches("`logout`").count();
    assert_eq!(
        logout_count, 1,
        "logout should not be duplicated after second --fix; found {logout_count}"
    );

    // login must also remain unduplicated
    let login_count = after_second.matches("`login`").count();
    assert_eq!(
        login_count, 1,
        "login should not be duplicated; found {login_count}"
    );
}

// Regression: fix_near_miss_headers used a small hardcoded pattern list and missed
// many real-world typos (singular forms, uncommon letter transpositions, etc.).
// Now uses Levenshtein distance ≤ 2 against a canonical list.
#[test]
fn fix_near_miss_handles_levenshtein_typos() {
    let tmp = TempDir::new().unwrap();
    let root = tmp.path();

    write_config(root, "specs", &["src"]);

    fs::create_dir_all(root.join("src/utils")).unwrap();
    fs::write(
        root.join("src/utils/helpers.ts"),
        "export function doStuff() {}\n",
    )
    .unwrap();

    // Spec with a near-miss header that the old code didn't cover:
    // "### Exporteed Functions" has edit distance 1 from "Exported Functions"
    // (extra 'e'), but didn't match any old hardcoded pattern.
    fs::create_dir_all(root.join("specs/utils")).unwrap();
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

Utility helpers.

## Public API

### Exporteed Functions

| Export | Description |
|--------|-------------|
| `doStuff` | does stuff |

## Change Log

| Date | Author | Change |
|------|--------|--------|
"#;
    fs::write(root.join("specs/utils/utils.spec.md"), spec).unwrap();

    specsync()
        .args(["check", "--fix", "--root", root.to_str().unwrap()])
        .assert()
        .success();

    let updated = fs::read_to_string(root.join("specs/utils/utils.spec.md")).unwrap();
    assert!(
        updated.contains("### Exported Functions"),
        "near-miss header should have been renamed to '### Exported Functions'"
    );
    assert!(
        !updated.contains("### Exporteed Functions"),
        "original near-miss header should be gone"
    );
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

// ─── Wildcard Re-export Integration Tests ───────────────────────────────

#[test]
fn wildcard_reexport_barrel_file_detected() {
    let tmp = TempDir::new().unwrap();
    let root = tmp.path();

    write_config(root, "specs", &["src"]);

    // Create a multi-file TypeScript project with a barrel (index.ts)
    fs::create_dir_all(root.join("src/utils")).unwrap();

    // helpers.ts — the real exports
    fs::write(
        root.join("src/utils/helpers.ts"),
        "export function formatDate() {}\nexport function parseUrl() {}\nexport const MAX_RETRIES = 3;\n",
    )
    .unwrap();

    // types.ts — type exports
    fs::write(
        root.join("src/utils/types.ts"),
        "export interface Config {}\nexport type Result = string;\n",
    )
    .unwrap();

    // index.ts — barrel file re-exporting everything
    fs::write(
        root.join("src/utils/index.ts"),
        "export * from './helpers';\nexport * from './types';\nexport function utilMain() {}\n",
    )
    .unwrap();

    // Spec pointing at the barrel file
    fs::create_dir_all(root.join("specs/utils")).unwrap();
    fs::write(
        root.join("specs/utils/utils.spec.md"),
        valid_spec("utils", &["src/utils/index.ts"]),
    )
    .unwrap();

    // check should detect the re-exported symbols as undocumented
    let output = specsync()
        .args(["check", "--root", root.to_str().unwrap()])
        .output()
        .unwrap();

    let stdout = String::from_utf8_lossy(&output.stdout);
    // The check should find undocumented exports from the barrel file
    assert!(
        stdout.contains("formatDate") || stdout.contains("parseUrl") || stdout.contains("utilMain"),
        "Expected check to detect wildcard re-exported symbols. Got:\n{stdout}"
    );
}

#[test]
fn wildcard_reexport_with_fix_adds_all_symbols() {
    let tmp = TempDir::new().unwrap();
    let root = tmp.path();

    write_config(root, "specs", &["src"]);

    fs::create_dir_all(root.join("src/utils")).unwrap();
    fs::write(
        root.join("src/utils/helpers.ts"),
        "export function helperA() {}\nexport function helperB() {}\n",
    )
    .unwrap();
    fs::write(
        root.join("src/utils/index.ts"),
        "export * from './helpers';\nexport function main() {}\n",
    )
    .unwrap();

    fs::create_dir_all(root.join("specs/utils")).unwrap();
    fs::write(
        root.join("specs/utils/utils.spec.md"),
        valid_spec("utils", &["src/utils/index.ts"]),
    )
    .unwrap();

    // Run --fix to auto-add all re-exported symbols
    specsync()
        .args(["check", "--fix", "--root", root.to_str().unwrap()])
        .assert()
        .success();

    let updated = fs::read_to_string(root.join("specs/utils/utils.spec.md")).unwrap();
    assert!(
        updated.contains("`helperA`"),
        "Expected helperA from wildcard re-export"
    );
    assert!(
        updated.contains("`helperB`"),
        "Expected helperB from wildcard re-export"
    );
    assert!(updated.contains("`main`"), "Expected main direct export");
}

#[test]
fn wildcard_namespace_reexport_detected() {
    let tmp = TempDir::new().unwrap();
    let root = tmp.path();

    write_config(root, "specs", &["src"]);

    fs::create_dir_all(root.join("src/lib")).unwrap();
    fs::write(
        root.join("src/lib/math.ts"),
        "export function add() {}\nexport function subtract() {}\n",
    )
    .unwrap();
    fs::write(
        root.join("src/lib/index.ts"),
        "export * as MathUtils from './math';\nexport function init() {}\n",
    )
    .unwrap();

    fs::create_dir_all(root.join("specs/lib")).unwrap();
    fs::write(
        root.join("specs/lib/lib.spec.md"),
        valid_spec("lib", &["src/lib/index.ts"]),
    )
    .unwrap();

    // check should detect MathUtils namespace and init
    let output = specsync()
        .args(["check", "--root", root.to_str().unwrap()])
        .output()
        .unwrap();

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        stdout.contains("MathUtils") || stdout.contains("init"),
        "Expected namespace re-export or direct export to be detected. Got:\n{stdout}"
    );
}

#[test]
fn wildcard_reexport_nested_barrel_only_one_level() {
    let tmp = TempDir::new().unwrap();
    let root = tmp.path();

    write_config(root, "specs", &["src"]);

    fs::create_dir_all(root.join("src/deep")).unwrap();

    // bottom.ts has the real exports
    fs::write(
        root.join("src/deep/bottom.ts"),
        "export function deepFunc() {}\n",
    )
    .unwrap();

    // middle.ts re-exports bottom
    fs::write(
        root.join("src/deep/middle.ts"),
        "export * from './bottom';\n",
    )
    .unwrap();

    // top.ts re-exports middle
    fs::write(
        root.join("src/deep/top.ts"),
        "export * from './middle';\nexport function topFunc() {}\n",
    )
    .unwrap();

    fs::create_dir_all(root.join("specs/deep")).unwrap();
    fs::write(
        root.join("specs/deep/deep.spec.md"),
        valid_spec("deep", &["src/deep/top.ts"]),
    )
    .unwrap();

    // Resolver only goes one level deep (no recursive resolver)
    // so deepFunc should NOT appear, but topFunc and middle's direct exports should
    let output = specsync()
        .args(["check", "--root", root.to_str().unwrap()])
        .output()
        .unwrap();

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        stdout.contains("topFunc"),
        "Expected topFunc to be found. Got:\n{stdout}"
    );
}

// ─── Hash caching ────────────────────────────────────────────────────────

#[test]
fn check_creates_hash_cache() {
    let tmp = TempDir::new().unwrap();
    let root = setup_minimal_project(&tmp);

    specsync()
        .args(["check", "--root", root.to_str().unwrap()])
        .assert()
        .success();

    let cache_path = root.join(".specsync/hashes.json");
    assert!(
        cache_path.exists(),
        "Expected .specsync/hashes.json to be created after check"
    );

    let content = fs::read_to_string(&cache_path).unwrap();
    let cache: serde_json::Value = serde_json::from_str(&content).unwrap();
    let hashes = cache["hashes"].as_object().unwrap();
    assert!(!hashes.is_empty(), "Expected hash cache to contain entries");
}

#[test]
fn check_skips_unchanged_specs() {
    let tmp = TempDir::new().unwrap();
    let root = setup_minimal_project(&tmp);

    // First run: creates cache
    specsync()
        .args(["check", "--root", root.to_str().unwrap()])
        .assert()
        .success();

    // Second run: should skip unchanged specs
    let output = specsync()
        .args(["check", "--root", root.to_str().unwrap()])
        .output()
        .unwrap();

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        stdout.contains("unchanged"),
        "Expected 'unchanged' message on second run. Got:\n{stdout}"
    );
}

#[test]
fn check_force_revalidates_all() {
    let tmp = TempDir::new().unwrap();
    let root = setup_minimal_project(&tmp);

    // First run: creates cache
    specsync()
        .args(["check", "--root", root.to_str().unwrap()])
        .assert()
        .success();

    // Second run with --force: should NOT skip
    let output = specsync()
        .args(["check", "--root", root.to_str().unwrap(), "--force"])
        .output()
        .unwrap();

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        !stdout.contains("Skipped"),
        "Expected no skip message with --force. Got:\n{stdout}"
    );
    assert!(
        stdout.contains("specs checked"),
        "Expected validation output with --force. Got:\n{stdout}"
    );
}

#[test]
fn check_revalidates_after_source_change() {
    let tmp = TempDir::new().unwrap();
    let root = setup_minimal_project(&tmp);

    // First run: creates cache
    specsync()
        .args(["check", "--root", root.to_str().unwrap()])
        .assert()
        .success();

    // Modify source file
    fs::write(
        root.join("src/auth/service.ts"),
        "export function login() {}\nexport function logout() {}\nexport function refresh() {}\n",
    )
    .unwrap();

    // Second run: should detect change and revalidate
    let output = specsync()
        .args(["check", "--root", root.to_str().unwrap()])
        .output()
        .unwrap();

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        !stdout.contains("All specs unchanged"),
        "Expected revalidation after source change. Got:\n{stdout}"
    );
}

// ─── Custom validation rules ────────────────────────────────────────────

/// Write a specsync.json with custom rules.
fn write_config_with_custom_rules(root: &std::path::Path, custom_rules_json: &str) {
    let config = format!(
        r#"{{
  "specsDir": "specs",
  "sourceDirs": ["src"],
  "requiredSections": ["Purpose", "Public API", "Invariants", "Behavioral Examples", "Error Cases", "Dependencies", "Change Log"],
  "excludeDirs": ["__tests__"],
  "excludePatterns": ["**/__tests__/**"],
  "customRules": {custom_rules_json}
}}"#
    );
    fs::write(root.join("specsync.json"), config).unwrap();
}

#[test]
fn custom_rule_require_section_triggers_warning() {
    let tmp = TempDir::new().unwrap();
    let root = tmp.path().to_path_buf();

    write_config_with_custom_rules(
        &root,
        r#"[{
            "name": "need-threat-model",
            "type": "require_section",
            "section": "Threat Model",
            "severity": "warning",
            "message": "Specs must include a Threat Model section"
        }]"#,
    );

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

    let output = specsync()
        .args(["check", "--root", root.to_str().unwrap(), "--force"])
        .output()
        .unwrap();
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        stdout.contains("Threat Model"),
        "Expected custom rule warning about Threat Model. Got:\n{stdout}"
    );
    assert!(
        stdout.contains("need-threat-model"),
        "Expected rule name in output. Got:\n{stdout}"
    );
}

#[test]
fn custom_rule_require_section_passes_when_section_exists() {
    let tmp = TempDir::new().unwrap();
    let root = tmp.path().to_path_buf();

    write_config_with_custom_rules(
        &root,
        r#"[{
            "name": "need-purpose",
            "type": "require_section",
            "section": "Purpose",
            "severity": "error"
        }]"#,
    );

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

    let output = specsync()
        .args(["check", "--root", root.to_str().unwrap(), "--force"])
        .output()
        .unwrap();
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        !stdout.contains("need-purpose"),
        "Purpose section exists — rule should NOT trigger. Got:\n{stdout}"
    );
}

#[test]
fn custom_rule_forbid_pattern_triggers_on_match() {
    let tmp = TempDir::new().unwrap();
    let root = tmp.path().to_path_buf();

    write_config_with_custom_rules(
        &root,
        r#"[{
            "name": "no-fixme",
            "type": "forbid_pattern",
            "pattern": "FIXME",
            "severity": "warning",
            "message": "Specs should not contain FIXME markers"
        }]"#,
    );

    fs::create_dir_all(root.join("src/auth")).unwrap();
    fs::write(
        root.join("src/auth/service.ts"),
        "export function login() {}\n",
    )
    .unwrap();
    fs::create_dir_all(root.join("specs/auth")).unwrap();
    // Add FIXME to the spec
    let mut spec = valid_spec("auth", &["src/auth/service.ts"]);
    spec.push_str("\nFIXME: need to update this\n");
    fs::write(root.join("specs/auth/auth.spec.md"), spec).unwrap();

    let output = specsync()
        .args(["check", "--root", root.to_str().unwrap(), "--force"])
        .output()
        .unwrap();
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        stdout.contains("FIXME"),
        "Expected forbid_pattern warning about FIXME. Got:\n{stdout}"
    );
}

#[test]
fn custom_rule_min_word_count_triggers_on_short_section() {
    let tmp = TempDir::new().unwrap();
    let root = tmp.path().to_path_buf();

    write_config_with_custom_rules(
        &root,
        r#"[{
            "name": "detailed-purpose",
            "type": "min_word_count",
            "section": "Purpose",
            "minWords": 50,
            "severity": "warning"
        }]"#,
    );

    fs::create_dir_all(root.join("src/auth")).unwrap();
    fs::write(
        root.join("src/auth/service.ts"),
        "export function login() {}\n",
    )
    .unwrap();
    fs::create_dir_all(root.join("specs/auth")).unwrap();
    // valid_spec has a very short Purpose section ("This module does something.")
    fs::write(
        root.join("specs/auth/auth.spec.md"),
        valid_spec("auth", &["src/auth/service.ts"]),
    )
    .unwrap();

    let output = specsync()
        .args(["check", "--root", root.to_str().unwrap(), "--force"])
        .output()
        .unwrap();
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        stdout.contains("detailed-purpose"),
        "Expected min_word_count warning. Got:\n{stdout}"
    );
}

#[test]
fn custom_rule_applies_to_filters_by_status() {
    let tmp = TempDir::new().unwrap();
    let root = tmp.path().to_path_buf();

    // Rule only applies to "stable" specs — our spec is "active", so it should NOT trigger
    write_config_with_custom_rules(
        &root,
        r#"[{
            "name": "stable-only-rule",
            "type": "require_section",
            "section": "Security Considerations",
            "severity": "error",
            "appliesTo": { "status": "stable" }
        }]"#,
    );

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

    let output = specsync()
        .args(["check", "--root", root.to_str().unwrap(), "--force"])
        .output()
        .unwrap();
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        !stdout.contains("stable-only-rule"),
        "Rule should NOT trigger for active specs. Got:\n{stdout}"
    );
}

#[test]
fn custom_rule_error_severity_causes_failure_in_strict() {
    let tmp = TempDir::new().unwrap();
    let root = tmp.path().to_path_buf();

    write_config_with_custom_rules(
        &root,
        r#"[{
            "name": "require-security",
            "type": "require_section",
            "section": "Security Review",
            "severity": "error",
            "message": "All specs must include a Security Review"
        }]"#,
    );

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

    specsync()
        .args([
            "check",
            "--root",
            root.to_str().unwrap(),
            "--force",
            "--enforcement",
            "strict",
        ])
        .assert()
        .failure();
}

#[test]
fn rules_command_lists_custom_rules() {
    let tmp = TempDir::new().unwrap();
    let root = tmp.path().to_path_buf();

    write_config_with_custom_rules(
        &root,
        r#"[{
            "name": "my-custom-rule",
            "type": "forbid_pattern",
            "pattern": "HACK",
            "severity": "warning",
            "message": "No HACK comments allowed"
        }]"#,
    );

    let output = specsync()
        .args(["rules", "--root", root.to_str().unwrap()])
        .output()
        .unwrap();
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        stdout.contains("my-custom-rule"),
        "Expected rule name in rules output. Got:\n{stdout}"
    );
    assert!(
        stdout.contains("forbid_pattern"),
        "Expected rule type in output. Got:\n{stdout}"
    );
    assert!(
        stdout.contains("HACK"),
        "Expected pattern in output. Got:\n{stdout}"
    );
}

// ─── Batch Operations ────────────────────────────────────────────────────

#[test]
fn score_all_format_table_outputs_headers() {
    let tmp = TempDir::new().unwrap();
    let root = setup_minimal_project(&tmp);

    specsync()
        .args(["score", "--all", "--format", "table", "--root"])
        .arg(&root)
        .assert()
        .success()
        .stdout(predicate::str::contains("Spec"))
        .stdout(predicate::str::contains("Score"))
        .stdout(predicate::str::contains("Grade"));
}

#[test]
fn score_all_format_csv_outputs_header_row() {
    let tmp = TempDir::new().unwrap();
    let root = setup_minimal_project(&tmp);

    specsync()
        .args(["score", "--all", "--format", "csv", "--root"])
        .arg(&root)
        .assert()
        .success()
        .stdout(predicate::str::contains(
            "spec,score,grade,frontmatter,sections,api,depth,freshness",
        ));
}

#[test]
fn score_all_format_csv_includes_summary_row() {
    let tmp = TempDir::new().unwrap();
    let root = setup_minimal_project(&tmp);

    specsync()
        .args(["score", "--all", "--format", "csv", "--root"])
        .arg(&root)
        .assert()
        .success()
        .stdout(predicate::str::contains("SUMMARY,"));
}

#[test]
fn score_format_table_without_all_flag_still_works() {
    let tmp = TempDir::new().unwrap();
    let root = setup_minimal_project(&tmp);

    // --all is optional; table output should work without it
    specsync()
        .args(["score", "--format", "table", "--root"])
        .arg(&root)
        .assert()
        .success()
        .stdout(predicate::str::contains("Grade"));
}

#[test]
fn generate_uncovered_flag_accepted() {
    let tmp = TempDir::new().unwrap();
    let root = setup_minimal_project(&tmp);

    // --uncovered is a no-op when all modules are specced
    specsync()
        .args(["generate", "--uncovered", "--root"])
        .arg(&root)
        .assert()
        .success();
}

#[test]
fn generate_batch_empty_list_skips_gracefully() {
    let tmp = TempDir::new().unwrap();
    let root = setup_minimal_project(&tmp);

    // Request a module that doesn't exist in coverage
    specsync()
        .args(["generate", "--batch", "nonexistent-module", "--root"])
        .arg(&root)
        .assert()
        .success()
        .stdout(
            predicate::str::contains("not found in coverage report")
                .or(predicate::str::contains("Nothing to generate")),
        );
}

#[test]
fn import_without_args_or_flags_shows_error() {
    let tmp = TempDir::new().unwrap();
    let root = tmp.path().to_path_buf();
    write_config(&root, "specs", &["src"]);

    specsync()
        .args(["import", "--root"])
        .arg(&root)
        .assert()
        .failure()
        .stderr(
            predicate::str::contains("SOURCE is required").or(predicate::str::contains("required")),
        );
}

#[test]
fn import_from_dir_imports_markdown_files() {
    let tmp = TempDir::new().unwrap();
    let root = tmp.path().to_path_buf();
    write_config(&root, "specs", &["src"]);
    fs::create_dir_all(root.join("docs")).unwrap();
    fs::create_dir_all(root.join("specs")).unwrap();

    // Write a simple markdown doc to import
    fs::write(
        root.join("docs/my-feature.md"),
        "# My Feature\n\nThis is a great feature.\n\n- [ ] Do something\n- [ ] Do another thing\n",
    )
    .unwrap();

    specsync()
        .args(["import", "--from-dir", "docs", "--root"])
        .arg(&root)
        .assert()
        .success()
        .stdout(predicate::str::contains("Batch Import"))
        .stdout(predicate::str::contains("1 imported").or(predicate::str::contains("imported")));
}

#[test]
fn import_from_dir_skips_existing_specs() {
    let tmp = TempDir::new().unwrap();
    let root = tmp.path().to_path_buf();
    write_config(&root, "specs", &["src"]);
    fs::create_dir_all(root.join("docs")).unwrap();

    // Pre-create the spec
    let spec_dir = root.join("specs/my-feature");
    fs::create_dir_all(&spec_dir).unwrap();
    fs::write(
        spec_dir.join("my-feature.spec.md"),
        valid_spec("my-feature", &[]),
    )
    .unwrap();

    // Write the same-named doc
    fs::write(
        root.join("docs/my-feature.md"),
        "# My Feature\n\nAlready exists.\n",
    )
    .unwrap();

    specsync()
        .args(["import", "--from-dir", "docs", "--root"])
        .arg(&root)
        .assert()
        .success()
        .stdout(predicate::str::contains("skipped"));
}

#[test]
fn import_from_dir_nonexistent_directory_errors() {
    let tmp = TempDir::new().unwrap();
    let root = tmp.path().to_path_buf();
    write_config(&root, "specs", &["src"]);
    fs::create_dir_all(root.join("specs")).unwrap();

    specsync()
        .args(["import", "--from-dir", "nonexistent-dir", "--root"])
        .arg(&root)
        .assert()
        .failure()
        .stderr(
            predicate::str::contains("Directory not found")
                .or(predicate::str::contains("not found")),
        );
}

// ─── specsync migrate ──────────────────────────────────────────────────

/// Set up a valid 3.x project with lifecycle_log in frontmatter.
fn setup_v3_project(root: &std::path::Path) {
    // v3-style config at root
    write_config(root, "specs", &["src"]);

    // Registry at root
    fs::write(
        root.join("specsync-registry.toml"),
        r#"[registry]
name = "test-project"

[[modules]]
name = "auth"
spec = "specs/auth/auth.spec.md"
"#,
    )
    .unwrap();

    // Source
    fs::create_dir_all(root.join("src/auth")).unwrap();
    fs::write(
        root.join("src/auth/service.ts"),
        "export function login() {}\nexport function logout() {}\n",
    )
    .unwrap();

    // Spec with lifecycle_log in frontmatter
    fs::create_dir_all(root.join("specs/auth")).unwrap();
    let spec = r#"---
module: auth
version: 1
status: active
files:
  - src/auth/service.ts
db_tables: []
depends_on: []
lifecycle_log:
  - "2026-04-01: draft → review"
  - "2026-04-05: review → stable"
---

# Auth

## Purpose

Authentication module.

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
"#;
    fs::write(root.join("specs/auth/auth.spec.md"), spec).unwrap();
}

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

/// Write a v4 TOML config under .specsync/config.toml.
fn write_toml_config(root: &std::path::Path, extra: &str) {
    fs::create_dir_all(root.join(".specsync")).unwrap();
    let config = format!("specs_dir = \"specs\"\nsource_dirs = [\"src\"]\n{extra}");
    fs::write(root.join(".specsync/config.toml"), config).unwrap();
    fs::write(root.join(".specsync/version"), "4.0.0").unwrap();
}

/// Set up a minimal v4 project (TOML config) with one source module and no spec.
fn setup_v4_unspecced(tmp: &TempDir, config_extra: &str) -> std::path::PathBuf {
    let root = tmp.path().to_path_buf();
    write_toml_config(&root, config_extra);
    fs::create_dir_all(root.join("src/billing")).unwrap();
    fs::write(
        root.join("src/billing/index.ts"),
        "export function charge() {}\nexport function refund() {}\n",
    )
    .unwrap();
    fs::create_dir_all(root.join("specs")).unwrap();
    root
}

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

// ─── Additional Custom Rules Tests ──────────────────────────────────────

#[test]
fn custom_rule_require_pattern_passes_when_present() {
    let tmp = TempDir::new().unwrap();
    let root = tmp.path().to_path_buf();

    write_config_with_custom_rules(
        &root,
        r#"[{
            "name": "require-version-table",
            "type": "require_pattern",
            "pattern": "\\| Date .* Change \\|",
            "severity": "error",
            "message": "Specs must have a changelog table"
        }]"#,
    );

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

    specsync()
        .args(["check", "--root", root.to_str().unwrap(), "--force"])
        .assert()
        .success();
}

#[test]
fn custom_rule_applies_to_filter_skips_non_matching_modules() {
    let tmp = TempDir::new().unwrap();
    let root = tmp.path().to_path_buf();

    write_config_with_custom_rules(
        &root,
        r#"[{
            "name": "auth-security",
            "type": "require_section",
            "section": "Security Review",
            "severity": "error",
            "appliesTo": { "module": "^auth" }
        }]"#,
    );

    fs::create_dir_all(root.join("src/utils")).unwrap();
    fs::write(
        root.join("src/utils/helpers.ts"),
        "export function helper() {}\n",
    )
    .unwrap();
    fs::create_dir_all(root.join("specs/utils")).unwrap();
    fs::write(
        root.join("specs/utils/utils.spec.md"),
        valid_spec("utils", &["src/utils/helpers.ts"]),
    )
    .unwrap();

    let output = specsync()
        .args(["check", "--root", root.to_str().unwrap(), "--force"])
        .output()
        .unwrap();
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        !stdout.contains("auth-security"),
        "Module filter ^auth should skip utils module. Got:\n{stdout}"
    );
}

// ─── #244: --fix --dry-run does not modify files ────────────────────────

#[test]
fn fix_dry_run_does_not_write_files() {
    let tmp = TempDir::new().unwrap();
    let root = tmp.path().to_path_buf();

    write_config(&root, "specs", &["src"]);
    fs::create_dir_all(root.join("specs/mymod")).unwrap();
    fs::create_dir_all(root.join("src/mymod")).unwrap();
    fs::write(
        root.join("src/mymod/index.ts"),
        "export function hello() {}\nexport function world() {}\n",
    )
    .unwrap();
    // Spec only documents 'hello', so --fix would add 'world'
    let spec_content = valid_spec("mymod", &["src/mymod/index.ts"]);
    let spec_with_hello = spec_content.replace(
        "| Function | Parameters | Returns | Description |",
        "| Function | Parameters | Returns | Description |\n| `hello` | | void | Says hello |",
    );
    fs::write(root.join("specs/mymod/mymod.spec.md"), &spec_with_hello).unwrap();
    // Also create companion requirements.md to suppress warning
    fs::write(root.join("specs/mymod/requirements.md"), "# Requirements\n").unwrap();

    let original = fs::read_to_string(root.join("specs/mymod/mymod.spec.md")).unwrap();

    specsync()
        .args([
            "check",
            "--fix",
            "--dry-run",
            "--root",
            root.to_str().unwrap(),
            "--force",
        ])
        .assert()
        .success();

    let after = fs::read_to_string(root.join("specs/mymod/mymod.spec.md")).unwrap();
    assert_eq!(original, after, "--dry-run should not modify spec files");
}

// ─── #244: --fix --backup creates backup files ────────���─────────────────

#[test]
fn fix_backup_creates_backup_dir() {
    let tmp = TempDir::new().unwrap();
    let root = tmp.path().to_path_buf();

    write_config(&root, "specs", &["src"]);
    fs::create_dir_all(root.join("specs/bmod")).unwrap();
    fs::create_dir_all(root.join("src/bmod")).unwrap();
    fs::write(
        root.join("src/bmod/index.ts"),
        "export function alpha() {}\nexport function beta() {}\n",
    )
    .unwrap();
    let spec_content = valid_spec("bmod", &["src/bmod/index.ts"]);
    let spec_with_alpha = spec_content.replace(
        "| Function | Parameters | Returns | Description |",
        "| Function | Parameters | Returns | Description |\n| `alpha` | | void | Alpha |",
    );
    fs::write(root.join("specs/bmod/bmod.spec.md"), &spec_with_alpha).unwrap();
    fs::write(root.join("specs/bmod/requirements.md"), "# Requirements\n").unwrap();

    specsync()
        .args([
            "check",
            "--fix",
            "--backup",
            "--root",
            root.to_str().unwrap(),
            "--force",
        ])
        .assert()
        .success();

    let backup_dir = root.join(".specsync/backup-fix");
    assert!(
        backup_dir.exists(),
        "--backup should create .specsync/backup-fix/"
    );
    assert!(
        backup_dir.join("specs/bmod/bmod.spec.md").exists(),
        "Backup should contain the original spec file"
    );
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
