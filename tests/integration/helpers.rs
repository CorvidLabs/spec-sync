// ─── Helpers ─────────────────────────────────────────────────────────────
use assert_cmd::Command;
use std::fs;
use tempfile::TempDir;

pub fn specsync() -> Command {
    let mut cmd = Command::cargo_bin("specsync").unwrap();
    cmd.env_remove("GITHUB_EVENT_NAME");
    cmd.env_remove("GITHUB_BASE_REF");
    cmd
}

pub fn valid_spec(module: &str, files: &[&str]) -> String {
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

pub fn write_config(root: &std::path::Path, specs_dir: &str, source_dirs: &[&str]) {
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

pub fn setup_minimal_project(tmp: &TempDir) -> std::path::PathBuf {
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

/// Send JSON-RPC requests to the MCP server via stdin and capture stdout.
pub fn mcp_request(
    root: &std::path::Path,
    requests: &[serde_json::Value],
) -> Vec<serde_json::Value> {
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

/// Write a specsync.json with custom rules.
pub fn write_config_with_custom_rules(root: &std::path::Path, custom_rules_json: &str) {
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

/// Set up a valid 3.x project with lifecycle_log in frontmatter.
pub fn setup_v3_project(root: &std::path::Path) {
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

/// Write a v4 TOML config under .specsync/config.toml.
pub fn write_toml_config(root: &std::path::Path, extra: &str) {
    fs::create_dir_all(root.join(".specsync")).unwrap();
    let config = format!("specs_dir = \"specs\"\nsource_dirs = [\"src\"]\n{extra}");
    fs::write(root.join(".specsync/config.toml"), config).unwrap();
    fs::write(root.join(".specsync/version"), "4.0.0").unwrap();
}

/// Set up a minimal v4 project (TOML config) with one source module and no spec.
pub fn setup_v4_unspecced(tmp: &TempDir, config_extra: &str) -> std::path::PathBuf {
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

/// Build a two-module project where `api` imports `db` without declaring it.
pub fn setup_undeclared_import_project(root: &std::path::Path) {
    write_config(root, "specs", &["src"]);
    fs::create_dir_all(root.join("src/api")).unwrap();
    fs::create_dir_all(root.join("src/db")).unwrap();
    fs::create_dir_all(root.join("specs/api")).unwrap();
    fs::create_dir_all(root.join("specs/db")).unwrap();
    fs::write(root.join("src/db/db.ts"), "export function query() {}\n").unwrap();
    fs::write(
        root.join("src/api/api.ts"),
        "import { query } from '../db/db';\nexport function handler() { return query(); }\n",
    )
    .unwrap();
    fs::write(
        root.join("specs/db/db.spec.md"),
        "---\nmodule: db\nversion: 1\nstatus: active\nfiles:\n  - src/db/db.ts\ndepends_on: []\n---\n# db\n## Purpose\np\n",
    )
    .unwrap();
    fs::write(
        root.join("specs/api/api.spec.md"),
        "---\nmodule: api\nversion: 1\nstatus: active\nfiles:\n  - src/api/api.ts\ndepends_on: []\n---\n# api\n## Purpose\np\n",
    )
    .unwrap();
}
