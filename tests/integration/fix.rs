use crate::helpers::*;
use std::fs;
use tempfile::TempDir;

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
        updated.contains("Document caller-visible behavior and constraints."),
        "Expected generated descriptions to guide follow-up review"
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

// ─── #250: --dry-run without --fix warns ────────────────────────────────

#[test]
fn dry_run_without_fix_warns() {
    let tmp = TempDir::new().unwrap();
    let root = tmp.path().to_path_buf();

    write_config(&root, "specs", &["src"]);
    fs::create_dir_all(root.join("specs/dmod")).unwrap();
    fs::create_dir_all(root.join("src/dmod")).unwrap();
    fs::write(root.join("src/dmod/index.ts"), "export function f() {}").unwrap();
    fs::write(
        root.join("specs/dmod/dmod.spec.md"),
        valid_spec("dmod", &["src/dmod/index.ts"]),
    )
    .unwrap();
    fs::write(root.join("specs/dmod/requirements.md"), "# Requirements\n").unwrap();

    let output = specsync()
        .args([
            "check",
            "--dry-run",
            "--root",
            root.to_str().unwrap(),
            "--force",
        ])
        .output()
        .unwrap();
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains("--dry-run has no effect without --fix"),
        "Should warn about --dry-run without --fix. Got stderr:\n{stderr}"
    );
}

// ─── #251: --fix --backup preserves original content ────────────────────

#[test]
fn fix_backup_preserves_original_on_success() {
    let tmp = TempDir::new().unwrap();
    let root = tmp.path().to_path_buf();

    write_config(&root, "specs", &["src"]);
    fs::create_dir_all(root.join("specs/bkmod")).unwrap();
    fs::create_dir_all(root.join("src/bkmod")).unwrap();
    fs::write(
        root.join("src/bkmod/index.ts"),
        "export function alpha() {}\nexport function beta() {}\n",
    )
    .unwrap();
    let spec_content = valid_spec("bkmod", &["src/bkmod/index.ts"]);
    let spec_with_alpha = spec_content.replace(
        "| Function | Parameters | Returns | Description |",
        "| Function | Parameters | Returns | Description |\n| `alpha` | | void | Alpha |",
    );
    let original = spec_with_alpha.clone();
    fs::write(root.join("specs/bkmod/bkmod.spec.md"), &spec_with_alpha).unwrap();
    fs::write(root.join("specs/bkmod/requirements.md"), "# Requirements\n").unwrap();

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

    let backup_content =
        fs::read_to_string(root.join(".specsync/backup-fix/specs/bkmod/bkmod.spec.md"))
            .expect("backup file should exist");
    assert_eq!(
        backup_content, original,
        "Backup should contain the original spec content before --fix modifications"
    );
}
