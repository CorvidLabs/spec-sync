use crate::helpers::*;
use assert_cmd::Command;
use predicates::prelude::*;
use std::fs;
use tempfile::TempDir;

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
        .arg("check")
        .arg("--root")
        .arg(&root)
        .arg("--force")
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

// ─── Marketplace action hardening ───────────────────────────────────────

#[test]
fn action_does_not_use_eval_for_user_arguments() {
    let action = fs::read_to_string("action.yml").expect("action.yml should be readable");

    assert!(
        !action.contains("eval "),
        "action.yml must not use eval for user-controlled inputs"
    );
    assert!(
        action.contains("CMD=(specsync check --force)"),
        "check command should be assembled as a bash argv array"
    );
    assert!(
        action.contains("\"${CMD[@]}\""),
        "check command should execute the argv array directly"
    );
    assert!(
        action.contains(r#"NORMALIZED_ARGS="${INPUT_ARGS//$'\r'/ }""#)
            && action.contains(r#"NORMALIZED_ARGS="${NORMALIZED_ARGS//$'\n'/ }""#)
            && action.contains(r#"NORMALIZED_ARGS="${NORMALIZED_ARGS//$'\t'/ }""#),
        "action should normalize multiline args before simple whitespace splitting"
    );
    assert!(
        action.contains("COMMENT_CMD=(specsync comment)"),
        "comment command should be assembled as a bash argv array"
    );
}

#[test]
fn action_validates_require_coverage_input() {
    let action = fs::read_to_string("action.yml").expect("action.yml should be readable");

    assert!(
        action.contains("require-coverage must be an integer from 0 to 100"),
        "action should reject invalid require-coverage values before command execution"
    );
    assert!(
        action.contains("[ \"$INPUT_REQUIRE_COVERAGE\" -gt 100 ]"),
        "action should enforce an upper coverage bound of 100"
    );
}

#[test]
fn action_requires_release_checksums() {
    let action = fs::read_to_string("action.yml").expect("action.yml should be readable");

    assert!(
        !action.contains("Verify checksum if available"),
        "checksums should not be optional for downloaded release binaries"
    );
    assert!(
        action.contains("curl -fsSL \"${BASE_URL}/${ARCHIVE}.sha256\""),
        "action should download release checksum files with fail-closed curl flags"
    );
    assert!(
        action.contains("shasum -a 256 -c \"${ARCHIVE}.sha256\""),
        "Unix archive checksums should be verified before extraction"
    );
}

// ─── Hands-on findings: v4 init-registry, draft visibility, fix routing ────

#[test]
fn init_registry_uses_v4_path_in_migrated_project() {
    let tmp = TempDir::new().unwrap();
    let root = tmp.path();

    // v4 project: config + version stamp under .specsync/
    fs::create_dir_all(root.join(".specsync")).unwrap();
    fs::write(
        root.join(".specsync/config.toml"),
        "specs_dir = \"specs\"\nsource_dirs = [\"src\"]\n",
    )
    .unwrap();
    fs::write(root.join(".specsync/version"), "4.0.0\n").unwrap();
    fs::create_dir_all(root.join("specs")).unwrap();

    specsync()
        .args(["init-registry", "--root", root.to_str().unwrap()])
        .assert()
        .success()
        .stdout(predicate::str::contains("Created .specsync/registry.toml"));

    assert!(root.join(".specsync/registry.toml").exists());
    assert!(
        !root.join("specsync-registry.toml").exists(),
        "must not recreate the legacy root-level registry in a v4 project"
    );

    // And the v4 registry must not re-trigger the legacy migration nag
    specsync()
        .args(["check", "--root", root.to_str().unwrap()])
        .assert()
        .success()
        .stderr(predicate::str::contains("Legacy 3.x layout").not());
}

#[test]
fn init_registry_keeps_legacy_path_for_legacy_project() {
    let tmp = TempDir::new().unwrap();
    let root = tmp.path();

    // Un-migrated 3.x project: root-level config, no version stamp
    write_config(root, "specs", &["src"]);
    fs::create_dir_all(root.join("specs")).unwrap();

    specsync()
        .args(["init-registry", "--root", root.to_str().unwrap()])
        .assert()
        .success()
        .stdout(predicate::str::contains("Created specsync-registry.toml"));

    assert!(root.join("specsync-registry.toml").exists());
}

#[test]
fn init_registry_is_idempotent_for_v4_registry() {
    let tmp = TempDir::new().unwrap();
    let root = tmp.path();

    fs::create_dir_all(root.join(".specsync")).unwrap();
    fs::write(root.join(".specsync/version"), "4.0.0\n").unwrap();
    fs::write(
        root.join(".specsync/registry.toml"),
        "[registry]\nname = \"existing\"\n",
    )
    .unwrap();

    specsync()
        .args(["init-registry", "--root", root.to_str().unwrap()])
        .assert()
        .success()
        .stdout(predicate::str::contains("already exists"));

    let content = fs::read_to_string(root.join(".specsync/registry.toml")).unwrap();
    assert!(content.contains("existing"));
}

#[test]
fn draft_spec_check_reports_skipped_validation() {
    let tmp = TempDir::new().unwrap();
    let root = setup_minimal_project(&tmp);

    // Draft spec missing several required sections
    let draft_spec = r#"---
module: draft-mod
version: 1
status: draft
files:
  - src/auth/service.ts
db_tables: []
depends_on: []
---

# Draft Mod

## Purpose

Work in progress.
"#;
    fs::create_dir_all(root.join("specs/draft-mod")).unwrap();
    fs::write(root.join("specs/draft-mod/draft-mod.spec.md"), draft_spec).unwrap();

    let assert = specsync()
        .args(["check", "--root", root.to_str().unwrap()])
        .assert()
        .success();
    let stdout = String::from_utf8_lossy(&assert.get_output().stdout).to_string();

    // The draft must not get misleading green checkmarks for skipped checks
    assert!(
        stdout.contains("Section validation skipped (status: draft)"),
        "expected explicit skip notice for sections, got:\n{stdout}"
    );
    assert!(
        stdout.contains("Export validation skipped (status: draft)"),
        "expected explicit skip notice for exports, got:\n{stdout}"
    );
    assert!(
        stdout.contains("draft spec(s) skipped section and export validation"),
        "expected summary hint about skipped drafts, got:\n{stdout}"
    );
    assert!(
        stdout.contains("set `status: active` to enable full checks"),
        "expected guidance to promote the draft, got:\n{stdout}"
    );
}

#[test]
fn active_spec_check_has_no_draft_skip_notice() {
    let tmp = TempDir::new().unwrap();
    let root = setup_minimal_project(&tmp);

    let assert = specsync()
        .args(["check", "--root", root.to_str().unwrap()])
        .assert()
        .success();
    let stdout = String::from_utf8_lossy(&assert.get_output().stdout).to_string();

    assert!(!stdout.contains("skipped (status: draft)"));
    assert!(!stdout.contains("draft spec(s) skipped"));
}

#[test]
fn fix_routes_functions_and_types_to_matching_tables() {
    let tmp = TempDir::new().unwrap();
    let root = tmp.path();

    write_config(root, "specs", &["src"]);

    // Source with two functions and one type
    fs::create_dir_all(root.join("src/mathx")).unwrap();
    fs::write(
        root.join("src/mathx/lib.rs"),
        "pub fn add(a: i32, b: i32) -> i32 { a + b }\n\
         pub fn subtract(a: i32, b: i32) -> i32 { a - b }\n\
         pub struct Calculator { pub precision: u8 }\n",
    )
    .unwrap();

    // Spec with empty Public API tables (template layout: Functions then Types)
    fs::create_dir_all(root.join("specs/mathx")).unwrap();
    fs::write(
        root.join("specs/mathx/mathx.spec.md"),
        valid_spec("mathx", &["src/mathx/lib.rs"]),
    )
    .unwrap();

    specsync()
        .args(["check", "--fix", "--root", root.to_str().unwrap()])
        .assert()
        .success();

    let updated = fs::read_to_string(root.join("specs/mathx/mathx.spec.md")).unwrap();

    // Extract each subsection to verify the rows landed in the right table
    let functions_start = updated.find("### Exported Functions").unwrap();
    let types_start = updated.find("### Exported Types").unwrap();
    let invariants_start = updated.find("## Invariants").unwrap();
    let functions_table = &updated[functions_start..types_start];
    let types_table = &updated[types_start..invariants_start];

    assert!(
        functions_table.contains("`add`") && functions_table.contains("`subtract`"),
        "functions must go to the Exported Functions table, got:\n{functions_table}"
    );
    assert!(
        !types_table.contains("`add`") && !types_table.contains("`subtract`"),
        "functions must NOT land in the Exported Types table, got:\n{types_table}"
    );
    assert!(
        types_table.contains("`Calculator`"),
        "types must go to the Exported Types table, got:\n{types_table}"
    );
    assert!(
        !functions_table.contains("`Calculator`"),
        "types must NOT land in the Exported Functions table, got:\n{functions_table}"
    );

    // The fixed spec passes a re-check without duplicate rows
    specsync()
        .args(["check", "--root", root.to_str().unwrap()])
        .assert()
        .success();
    let recheck = fs::read_to_string(root.join("specs/mathx/mathx.spec.md")).unwrap();
    assert_eq!(recheck.matches("`add`").count(), 1);
}

#[test]
fn failing_frontmatter_shows_negated_label() {
    let tmp = TempDir::new().unwrap();
    let root = tmp.path();

    write_config(root, "specs", &["src"]);
    fs::create_dir_all(root.join("src")).unwrap();
    fs::write(root.join("src/lib.rs"), "pub fn a() {}").unwrap();

    // Spec missing required frontmatter fields (no version/status)
    fs::create_dir_all(root.join("specs/bad")).unwrap();
    fs::write(
        root.join("specs/bad/bad.spec.md"),
        "---\nmodule: bad\nfiles:\n  - src/lib.rs\n---\n\n# Bad\n\n## Purpose\n\nx\n",
    )
    .unwrap();

    let assert = specsync()
        .args(["check", "--root", root.to_str().unwrap()])
        .assert();
    let stdout = String::from_utf8_lossy(&assert.get_output().stdout).to_string();

    assert!(
        stdout.contains("✗ Frontmatter invalid"),
        "failing frontmatter must show a negated label, got:\n{stdout}"
    );
    assert!(
        !stdout.contains("✗ Frontmatter valid"),
        "must not pair ✗ with a non-negated label, got:\n{stdout}"
    );
}

#[test]
fn check_unmatched_spec_filter_errors_without_contradiction() {
    let tmp = TempDir::new().unwrap();
    let root = setup_minimal_project(&tmp);

    let assert = specsync()
        .args(["check", "nosuchmodule", "--root", root.to_str().unwrap()])
        .assert()
        .failure()
        .code(1);
    let output = assert.get_output();
    let stdout = String::from_utf8_lossy(&output.stdout).to_string();
    let stderr = String::from_utf8_lossy(&output.stderr).to_string();

    assert!(
        stderr.contains("No specs matched"),
        "expected unmatched-filter warning, got stderr:\n{stderr}"
    );
    assert!(
        !stdout.contains("No spec files found"),
        "must not claim no spec files exist when specs are present, got:\n{stdout}"
    );
}

#[test]
fn nonexistent_root_errors_with_nonzero_exit() {
    specsync()
        .args(["check", "--root", "/nonexistent/path/for/specsync"])
        .assert()
        .failure()
        .code(2)
        .stderr(predicate::str::contains("does not exist"));
}

// ─── Hands-on findings round 2 ───────────────────────────────────────────

/// `specsync new` in a single-source-file project (e.g. a fresh cargo crate
/// with only src/lib.rs) must auto-detect that file even though its name
/// doesn't match the module — the README quickstart promises exactly this.
#[test]
fn new_auto_detects_single_source_file() {
    let tmp = TempDir::new().unwrap();
    let root = tmp.path();

    write_config(root, "specs", &["src"]);
    fs::create_dir_all(root.join("src")).unwrap();
    fs::write(
        root.join("src/lib.rs"),
        "/// Says hello.\npub fn greet(name: &str) -> String { format!(\"Hello, {name}!\") }\n",
    )
    .unwrap();

    specsync()
        .args(["new", "greeter", "--root", root.to_str().unwrap()])
        .assert()
        .success()
        .stdout(predicate::str::contains("Auto-detected 1 source file(s)"));

    let spec = fs::read_to_string(root.join("specs/greeter/greeter.spec.md")).unwrap();
    assert!(
        spec.contains("- src/lib.rs"),
        "expected src/lib.rs in frontmatter files, got:\n{spec}"
    );
    assert!(
        spec.contains("`greet`"),
        "expected pre-populated `greet` export, got:\n{spec}"
    );
    assert!(
        !spec.contains("files: []"),
        "files must not be empty, got:\n{spec}"
    );

    // The quickstart flow: the generated spec must pass `specsync check`
    specsync()
        .args(["check", "--root", root.to_str().unwrap()])
        .assert()
        .success();
}

/// When nothing can be auto-detected (multiple source files, none matching),
/// `new` must say so instead of silently writing a spec that fails validation.
#[test]
fn new_warns_when_no_source_files_match() {
    let tmp = TempDir::new().unwrap();
    let root = tmp.path();

    write_config(root, "specs", &["src"]);
    fs::create_dir_all(root.join("src")).unwrap();
    fs::write(root.join("src/alpha.rs"), "pub fn a() {}").unwrap();
    fs::write(root.join("src/beta.rs"), "pub fn b() {}").unwrap();

    specsync()
        .args(["new", "gamma", "--root", root.to_str().unwrap()])
        .assert()
        .success()
        .stderr(predicate::str::contains("No source files matched"));
}

/// `scaffold` gets the same single-source-file fallback as `new`.
#[test]
fn scaffold_auto_detects_single_source_file() {
    let tmp = TempDir::new().unwrap();
    let root = tmp.path();

    write_config(root, "specs", &["src"]);
    fs::create_dir_all(root.join("src")).unwrap();
    fs::write(root.join("src/lib.rs"), "pub fn greet() {}").unwrap();

    specsync()
        .args(["scaffold", "greeter", "--root", root.to_str().unwrap()])
        .assert()
        .success();

    let spec = fs::read_to_string(root.join("specs/greeter/greeter.spec.md")).unwrap();
    assert!(
        spec.contains("src/lib.rs"),
        "expected src/lib.rs in scaffolded spec, got:\n{spec}"
    );
}

/// `new`/`add-spec`/`scaffold` must refuse a module name that would escape the project
/// (path traversal) rather than writing spec/companion files to an arbitrary location.
#[test]
fn scaffold_rejects_module_name_path_traversal() {
    let tmp = TempDir::new().unwrap();
    let root = tmp.path();
    write_config(root, "specs", &["src"]);

    for sub in ["new", "add-spec", "scaffold"] {
        let output = specsync()
            .arg(sub)
            .arg("../../escape/evil")
            .arg("--root")
            .arg(root)
            .output()
            .unwrap();
        assert!(
            !output.status.success(),
            "`{sub}` with a traversal module name must fail loud"
        );
        let stderr = String::from_utf8_lossy(&output.stderr);
        assert!(
            stderr.contains("invalid module name"),
            "`{sub}` should report an invalid module name; stderr: {stderr}"
        );
    }

    // The guard runs before any filesystem write, so nothing escaped the project root.
    assert!(
        !root.parent().unwrap().join("escape").exists(),
        "a spec/companion escaped the project root"
    );
}

/// The original repro: a spec with a warning gets recorded in the hash cache,
/// then `check --fix` (without --force) reports "All specs unchanged" and
/// fixes nothing. An explicit --fix must never be a silent no-op.
#[test]
fn fix_runs_even_when_cache_marks_specs_unchanged() {
    let tmp = TempDir::new().unwrap();
    let root = tmp.path();

    write_config(root, "specs", &["src"]);
    fs::create_dir_all(root.join("src/utils")).unwrap();
    fs::write(
        root.join("src/utils/helpers.ts"),
        "export function documented() {}\nexport function undocumented() {}\n",
    )
    .unwrap();

    fs::create_dir_all(root.join("specs/utils")).unwrap();
    let spec = valid_spec("utils", &["src/utils/helpers.ts"]).replace(
        "| Function | Parameters | Returns | Description |\n|----------|-----------|---------|-------------|",
        "| Function | Parameters | Returns | Description |\n|----------|-----------|---------|-------------|\n| `documented` | — | void | Documented helper |",
    );
    fs::write(root.join("specs/utils/utils.spec.md"), spec).unwrap();

    // Failing strict run — this records the spec (and its sources) in the
    // hash cache even though it produced a warning
    specsync()
        .args(["check", "--strict", "--root", root.to_str().unwrap()])
        .assert()
        .failure();

    // Sanity: without --fix/--force/--strict the cache now skips the spec
    let assert = specsync()
        .args(["check", "--root", root.to_str().unwrap()])
        .assert()
        .success();
    let stdout = String::from_utf8_lossy(&assert.get_output().stdout).to_string();
    assert!(
        stdout.contains("All specs unchanged"),
        "expected the cache to skip the unchanged spec, got:\n{stdout}"
    );

    // --fix without --force must actually fix, not report "All specs unchanged"
    let assert = specsync()
        .args(["check", "--fix", "--root", root.to_str().unwrap()])
        .assert()
        .success();
    let stdout = String::from_utf8_lossy(&assert.get_output().stdout).to_string();
    assert!(
        !stdout.contains("All specs unchanged"),
        "--fix must bypass the hash cache, got:\n{stdout}"
    );

    let updated = fs::read_to_string(root.join("specs/utils/utils.spec.md")).unwrap();
    assert!(
        updated.contains("`undocumented`"),
        "--fix must add the undocumented export, got:\n{updated}"
    );

    // And all exports are documented on a forced re-check
    let assert = specsync()
        .args(["check", "--force", "--root", root.to_str().unwrap()])
        .assert()
        .success();
    let stdout = String::from_utf8_lossy(&assert.get_output().stdout).to_string();
    assert!(
        stdout.contains("2/2 exports documented"),
        "expected full export coverage after --fix, got:\n{stdout}"
    );
}

/// A symbol documented in a hand-written table under a bare heading like
/// "### Functions" (not on the export-header allowlist) must not be added a
/// second time by --fix. The bare heading is promoted to "### Exported
/// Functions" so the existing rows become the export table.
#[test]
fn fix_does_not_duplicate_symbols_documented_under_bare_headings() {
    let tmp = TempDir::new().unwrap();
    let root = tmp.path();

    write_config(root, "specs", &["src"]);
    fs::create_dir_all(root.join("src/greeter")).unwrap();
    fs::write(root.join("src/greeter/lib.rs"), "pub fn greet() {}\n").unwrap();

    fs::create_dir_all(root.join("specs/greeter")).unwrap();
    let spec = r#"---
module: greeter
version: 1
status: active
files:
  - src/greeter/lib.rs
db_tables: []
depends_on: []
---

# Greeter

## Purpose

Greets people.

## Public API

### Functions

| Function | Description |
|----------|-------------|
| `greet` | Says hello |

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
    fs::write(root.join("specs/greeter/greeter.spec.md"), spec).unwrap();

    specsync()
        .args(["check", "--fix", "--root", root.to_str().unwrap()])
        .assert()
        .success();

    let updated = fs::read_to_string(root.join("specs/greeter/greeter.spec.md")).unwrap();
    assert_eq!(
        updated.matches("`greet`").count(),
        1,
        "greet must not be duplicated, got:\n{updated}"
    );
    assert!(
        updated.contains("### Exported Functions"),
        "bare heading must be promoted to an export header, got:\n{updated}"
    );

    // The promoted table makes the export count as documented on re-check
    let assert = specsync()
        .args(["check", "--force", "--root", root.to_str().unwrap()])
        .assert()
        .success();
    let stdout = String::from_utf8_lossy(&assert.get_output().stdout).to_string();
    assert!(
        stdout.contains("1/1 exports documented"),
        "expected greet to count as documented after promotion, got:\n{stdout}"
    );
}

/// The summary's warning count must match the number of ⚠ lines printed —
/// the partial "N/M exports documented" line is counted as a warning, so it
/// must render as one (previously it rendered as a green ✓).
#[test]
fn warning_count_matches_printed_warning_lines() {
    let tmp = TempDir::new().unwrap();
    let root = tmp.path();

    write_config(root, "specs", &["src"]);
    fs::create_dir_all(root.join("src/utils")).unwrap();
    fs::write(
        root.join("src/utils/helpers.ts"),
        "export function documented() {}\nexport function undocumented() {}\n",
    )
    .unwrap();
    fs::create_dir_all(root.join("specs/utils")).unwrap();
    let spec = valid_spec("utils", &["src/utils/helpers.ts"]).replace(
        "| Function | Parameters | Returns | Description |\n|----------|-----------|---------|-------------|",
        "| Function | Parameters | Returns | Description |\n|----------|-----------|---------|-------------|\n| `documented` | — | void | Documented helper |",
    );
    fs::write(root.join("specs/utils/utils.spec.md"), spec).unwrap();

    let assert = specsync()
        .args(["check", "--root", root.to_str().unwrap()])
        .assert()
        .success();
    let stdout = String::from_utf8_lossy(&assert.get_output().stdout).to_string();

    // The partial export-coverage summary must render as a ⚠ line
    assert!(
        stdout.contains("⚠ 1/2 exports documented"),
        "partial export coverage must print as a warning, got:\n{stdout}"
    );

    // Extract N from the "… N warning(s) …" summary and compare with the
    // number of ⚠ lines actually printed
    let counted: usize = stdout
        .lines()
        .find_map(|line| {
            let idx = line.find(" warning(s)")?;
            line[..idx].rsplit(' ').next()?.parse().ok()
        })
        .expect("summary line with warning count");
    assert_eq!(
        stdout.matches('⚠').count(),
        counted,
        "warning count must match printed ⚠ lines, got:\n{stdout}"
    );
}

#[test]
fn incremental_check_detects_schema_drift_after_migration_edit() {
    // Regression (H2): the default incremental `check` must re-validate when a
    // schema/migration file changes, even if no spec's own files changed.
    // Previously a migration that dropped a documented column was silently
    // skipped ("nothing to validate") — a false-PASS only `--force` caught.
    let tmp = TempDir::new().unwrap();
    let root = tmp.path().to_path_buf();

    // schemaDir + strict enforcement: a schema-column ERROR gates the exit code
    // WITHOUT the --strict flag (which would itself bypass the cache we test).
    fs::write(
        root.join("specsync.json"),
        r#"{
  "specsDir": "specs",
  "sourceDirs": ["src"],
  "schemaDir": "db/migrations",
  "enforcement": "strict",
  "requiredSections": ["Purpose", "Public API", "Invariants", "Behavioral Examples", "Error Cases", "Dependencies", "Change Log"],
  "excludeDirs": ["__tests__"]
}"#,
    )
    .unwrap();

    // Source with no exports keeps Public API validation trivially clean.
    fs::create_dir_all(root.join("src")).unwrap();
    fs::write(root.join("src/msg.ts"), "const internal = 1;\n").unwrap();

    // Migration initially defines both columns.
    fs::create_dir_all(root.join("db/migrations")).unwrap();
    let migration = root.join("db/migrations/001_init.sql");
    fs::write(
        &migration,
        "CREATE TABLE messages (\n  id INTEGER PRIMARY KEY,\n  content TEXT NOT NULL\n);\n",
    )
    .unwrap();

    // Spec documents both columns, so the first run is clean.
    fs::create_dir_all(root.join("specs/msg")).unwrap();
    fs::write(
        root.join("specs/msg/msg.spec.md"),
        r#"---
module: msg
version: 1
status: active
files:
  - src/msg.ts
db_tables:
  - messages
depends_on: []
---

# Msg

## Purpose
Messaging.

## Public API

### Schema: messages

| Column | Type | Constraints |
|--------|------|-------------|
| `id` | INTEGER | PRIMARY KEY |
| `content` | TEXT | NOT NULL |

## Invariants
1. Valid.

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
"#,
    )
    .unwrap();

    // First run: clean, and it writes the hash cache (incl. the migration file).
    specsync()
        .arg("check")
        .arg("--root")
        .arg(&root)
        .assert()
        .success();

    // Drop the documented `content` column. The spec's own files are unchanged,
    // so the naive incremental skip would (wrongly) pass.
    fs::write(
        &migration,
        "CREATE TABLE messages (\n  id INTEGER PRIMARY KEY\n);\n",
    )
    .unwrap();

    // Second run on the default path (no --force / no --strict flag) must catch
    // the drift instead of reporting "nothing to validate".
    specsync()
        .arg("check")
        .arg("--root")
        .arg(&root)
        .assert()
        .failure()
        .stdout(predicate::str::contains(
            "documented in spec but not found in migrations",
        ))
        .stdout(predicate::str::contains("nothing to validate").not());
}

#[test]
fn check_gates_on_unreadable_schema_file() {
    // Regression: an unreadable (non-UTF-8) migration was silently skipped by
    // build_schema, so its tables vanished and db_tables validation became a
    // no-op — check passed green even under strict. It must now be a hard error.
    let tmp = TempDir::new().unwrap();
    let root = tmp.path();
    // enforcement: strict so a validation error gates the exit code without the
    // --strict flag (mirrors the schema-drift test above).
    fs::write(
        root.join(".specsync.toml"),
        "specs_dir = \"specs\"\nsource_dirs = [\"src\"]\nschema_dir = \"db/migrations\"\nenforcement = \"strict\"\n",
    )
    .unwrap();
    fs::create_dir_all(root.join("src")).unwrap();
    fs::write(root.join("src/a.rs"), "pub fn f() {}\n").unwrap();
    fs::create_dir_all(root.join("db/migrations")).unwrap();
    let mut bad = b"CREATE TABLE users (id INTEGER);\n".to_vec();
    bad.push(0xFF);
    fs::write(root.join("db/migrations/001_init.sql"), bad).unwrap();
    fs::create_dir_all(root.join("specs/m")).unwrap();
    fs::write(
        root.join("specs/m/m.spec.md"),
        "---\nmodule: m\nversion: 1\nstatus: active\nfiles:\n  - src/a.rs\ndb_tables:\n  - users\n---\n# m\n## Purpose\np\n",
    )
    .unwrap();

    // The unreadable migration is surfaced as a hard error AND gates the exit.
    specsync()
        .current_dir(root)
        .arg("check")
        .assert()
        .failure()
        .stdout(predicate::str::contains(
            "could not be read as UTF-8; DB schema",
        ));
}

#[test]
fn check_no_schema_error_for_readable_migration() {
    // No false positive: a readable migration produces no schema-read error.
    let tmp = TempDir::new().unwrap();
    let root = tmp.path();
    fs::write(
        root.join(".specsync.toml"),
        "specs_dir = \"specs\"\nsource_dirs = [\"src\"]\nschema_dir = \"db/migrations\"\n",
    )
    .unwrap();
    fs::create_dir_all(root.join("src")).unwrap();
    fs::write(root.join("src/a.rs"), "pub fn f() {}\n").unwrap();
    fs::create_dir_all(root.join("db/migrations")).unwrap();
    fs::write(
        root.join("db/migrations/001_init.sql"),
        "CREATE TABLE users (id INTEGER);\n",
    )
    .unwrap();
    fs::create_dir_all(root.join("specs/m")).unwrap();
    fs::write(
        root.join("specs/m/m.spec.md"),
        "---\nmodule: m\nversion: 1\nstatus: active\nfiles:\n  - src/a.rs\ndb_tables:\n  - users\n---\n# m\n## Purpose\np\n",
    )
    .unwrap();

    specsync()
        .current_dir(root)
        .arg("check")
        .assert()
        .stdout(predicate::str::contains("could not be read as UTF-8; DB schema").not());
}

// ─── config: multi-line TOML arrays are not corrupted ────────────────────

#[test]
fn migrate_preserves_multiline_toml_array_config() {
    // Regression (data loss): a formatter-style multi-line array in a legacy
    // `.specsync.toml` used to parse as `["["]`, and `migrate` (which reads then
    // rewrites the config) would persist that corruption — silently discarding the
    // user's real source_dirs/exclude_patterns.
    let tmp = TempDir::new().unwrap();
    let root = tmp.path();
    fs::create_dir_all(root.join("src")).unwrap();
    fs::create_dir_all(root.join("lib")).unwrap();
    fs::write(root.join("src/a.ts"), "export function a() {}\n").unwrap();
    fs::write(root.join("lib/b.ts"), "export function b() {}\n").unwrap();
    fs::write(
        root.join(".specsync.toml"),
        "specs_dir = \"specs\"\n\
         source_dirs = [\n  \"src\",\n  \"lib\",\n]\n\
         exclude_patterns = [\n  \"**/*.test.ts\",\n]\n",
    )
    .unwrap();

    specsync()
        .current_dir(root)
        .args(["migrate", "--no-backup"])
        .assert()
        .success();

    let migrated = fs::read_to_string(root.join(".specsync/config.toml")).unwrap();
    assert!(
        migrated.contains("\"src\"") && migrated.contains("\"lib\""),
        "both source dirs must survive migration:\n{migrated}"
    );
    assert!(
        !migrated.contains("[\"[\""),
        "config must not be corrupted to a bogus '[' entry:\n{migrated}"
    );
    assert!(
        migrated.contains("**/*.test.ts"),
        "exclude_patterns must survive migration:\n{migrated}"
    );
}

#[test]
fn migrate_preserves_parse_mode_and_modules() {
    // Regression (#2, data loss): config_to_toml used to silently drop parseMode
    // and modules, so migrating a 3.x project lost them. They must survive.
    let tmp = TempDir::new().unwrap();
    let root = tmp.path();
    fs::create_dir_all(root.join("src")).unwrap();
    fs::write(root.join("src/a.ts"), "export function a() {}\n").unwrap();
    fs::write(
        root.join("specsync.json"),
        r#"{
  "specsDir": "specs",
  "sourceDirs": ["src"],
  "parseMode": "ast",
  "modules": {
    "core": { "files": ["src/a.ts"], "dependsOn": ["util"] },
    "util": { "files": ["src/u.ts"] }
  }
}
"#,
    )
    .unwrap();

    specsync()
        .current_dir(root)
        .args(["migrate", "--no-backup"])
        .assert()
        .success();

    let migrated = fs::read_to_string(root.join(".specsync/config.toml")).unwrap();
    assert!(
        migrated.contains("parse_mode = \"ast\""),
        "parse_mode must survive migration:\n{migrated}"
    );
    assert!(
        migrated.contains("[modules.\"core\"]") && migrated.contains("[modules.\"util\"]"),
        "module definitions must survive migration:\n{migrated}"
    );
    assert!(
        migrated.contains("depends_on = [\"util\"]"),
        "module depends_on must survive migration:\n{migrated}"
    );
}

#[test]
fn migrate_refuses_config_with_custom_rules() {
    // Regression (#2, data loss): customRules cannot be represented in config.toml,
    // so migrate must REFUSE (exit 1) and leave specsync.json intact rather than
    // silently drop a security rule and delete the source.
    let tmp = TempDir::new().unwrap();
    let root = tmp.path();
    fs::create_dir_all(root.join("src")).unwrap();
    fs::write(root.join("src/a.ts"), "export function a() {}\n").unwrap();
    let original = r#"{
  "specsDir": "specs",
  "sourceDirs": ["src"],
  "customRules": [
    { "name": "threat-model", "type": "require_section", "section": "Threat Model", "severity": "error" }
  ]
}
"#;
    fs::write(root.join("specsync.json"), original).unwrap();

    specsync()
        .current_dir(root)
        .arg("migrate")
        .assert()
        .failure();

    // The original config is preserved byte-for-byte and no lossy TOML was written.
    assert_eq!(
        fs::read_to_string(root.join("specsync.json")).unwrap(),
        original,
        "specsync.json must be preserved unchanged"
    );
    assert!(
        !root.join(".specsync/config.toml").exists(),
        "no lossy config.toml should be written on refusal"
    );
}

#[test]
fn migrate_refuses_custom_rules_in_coexisting_v4_json() {
    // Regression (#2 review): migrate converts one source but DELETES the other
    // legacy configs unconverted. A higher-precedence .specsync/config.json with
    // customRules (alongside a clean root specsync.json) must still be caught — it
    // is the runtime-active config and would otherwise be silently destroyed.
    let tmp = TempDir::new().unwrap();
    let root = tmp.path();
    fs::create_dir_all(root.join("src")).unwrap();
    fs::create_dir_all(root.join(".specsync")).unwrap();
    fs::write(root.join("src/a.ts"), "export function a() {}\n").unwrap();
    let root_json =
        "{ \"specsDir\": \"specs\", \"sourceDirs\": [\"src\"], \"enforcement\": \"warn\" }\n";
    let v4_json = "{ \"specsDir\": \"specs\", \"sourceDirs\": [\"src\"], \"enforcement\": \"strict\", \"customRules\": [ { \"name\": \"threat-model\", \"type\": \"require_section\", \"section\": \"Threat Model\", \"severity\": \"error\" } ] }\n";
    fs::write(root.join("specsync.json"), root_json).unwrap();
    fs::write(root.join(".specsync/config.json"), v4_json).unwrap();

    specsync()
        .current_dir(root)
        .arg("migrate")
        .assert()
        .failure();

    // Both source configs preserved; no lossy config.toml written.
    assert_eq!(
        fs::read_to_string(root.join("specsync.json")).unwrap(),
        root_json
    );
    assert_eq!(
        fs::read_to_string(root.join(".specsync/config.json")).unwrap(),
        v4_json
    );
    assert!(!root.join(".specsync/config.toml").exists());
}

#[test]
fn migrate_preserves_explicitly_empty_arrays() {
    // Regression (#2 re-review): an explicit requiredSections/excludeDirs/
    // excludePatterns = [] (opting out) used to silently revert to the non-empty
    // defaults on migrate, flipping check results. They must survive as [].
    let tmp = TempDir::new().unwrap();
    let root = tmp.path();
    fs::create_dir_all(root.join("src")).unwrap();
    fs::write(root.join("src/a.ts"), "export function a() {}\n").unwrap();
    fs::write(
        root.join("specsync.json"),
        r#"{ "specsDir": "specs", "sourceDirs": ["src"], "requiredSections": [], "excludeDirs": [], "excludePatterns": [] }"#,
    )
    .unwrap();

    specsync()
        .current_dir(root)
        .args(["migrate", "--no-backup"])
        .assert()
        .success();

    let migrated = fs::read_to_string(root.join(".specsync/config.toml")).unwrap();
    assert!(
        migrated.contains("required_sections = []"),
        "explicit empty required_sections must be written, not omitted:\n{migrated}"
    );
    assert!(
        migrated.contains("exclude_patterns = []"),
        "explicit empty exclude_patterns must be written:\n{migrated}"
    );
    assert!(
        migrated.contains("exclude_dirs = []"),
        "explicit empty exclude_dirs must be written:\n{migrated}"
    );
}
