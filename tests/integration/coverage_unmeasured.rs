//! #582 — coverage over an empty denominator must report nothing measured.
//!
//! A project whose `source_dirs` yields zero source files, or whose source
//! files hold zero lines, has no coverage percentage. Reporting one as `100`
//! is the campaign's defect class in its purest form: a category is empty for
//! want of INPUT, and the code reads that as want of PROBLEMS. The number ends
//! up on a badge, in a PR comment, and in an agent's context, all saying the
//! project is fully specced when nothing was ever looked at.
//!
//! Every command and every format is asserted here, because #562 was fixed in
//! `src/output.rs` alone while eight other sites kept printing 100%.

use crate::helpers::*;
use predicates::prelude::*;
use std::fs;
use tempfile::TempDir;

/// Every format flag the coverage-reporting commands accept.
const FORMATS: [&str; 6] = ["text", "json", "markdown", "github", "table", "csv"];

/// Every command that renders a coverage figure.
const COMMANDS: [&str; 4] = ["check", "coverage", "report", "comment"];

/// A project with a spec but ZERO source files: `src/` holds only a file
/// discovery does not treat as source, so the spec is valid and `check`
/// passes. This is the tree CI sees as green.
fn zero_source_project(root: &std::path::Path) {
    write_config(root, "specs", &["src"]);
    fs::create_dir_all(root.join("src/auth")).unwrap();
    fs::write(root.join("src/auth/NOTES.md"), "notes\n").unwrap();
    fs::create_dir_all(root.join("specs/auth")).unwrap();
    fs::write(
        root.join("specs/auth/auth.spec.md"),
        valid_spec("auth", &["src/auth/NOTES.md"]),
    )
    .unwrap();
}

/// A project with one source file that has ZERO lines. Files are measured,
/// lines are not.
fn zero_loc_project(root: &std::path::Path) {
    write_config(root, "specs", &["src"]);
    fs::create_dir_all(root.join("src/auth")).unwrap();
    fs::write(root.join("src/auth/service.ts"), "").unwrap();
    fs::create_dir_all(root.join("specs/auth")).unwrap();
    fs::write(
        root.join("specs/auth/auth.spec.md"),
        valid_spec("auth", &["src/auth/service.ts"]),
    )
    .unwrap();
}

/// A healthy project: two source files, one of them specced — 50% coverage.
fn healthy_project(root: &std::path::Path) {
    write_config(root, "specs", &["src"]);
    fs::create_dir_all(root.join("src/auth")).unwrap();
    fs::create_dir_all(root.join("src/billing")).unwrap();
    fs::write(
        root.join("src/auth/service.ts"),
        "export function login(user: string): boolean {\n  return user.length > 0;\n}\n",
    )
    .unwrap();
    fs::write(
        root.join("src/billing/charge.ts"),
        "export function charge(amount: number): number {\n  return amount * 2;\n}\n",
    )
    .unwrap();
    fs::create_dir_all(root.join("specs/auth")).unwrap();
    fs::write(
        root.join("specs/auth/auth.spec.md"),
        valid_spec("auth", &["src/auth/service.ts"]),
    )
    .unwrap();
}

fn run(root: &std::path::Path, args: &[&str]) -> (String, String, i32) {
    let output = specsync()
        .args(args)
        .arg("--root")
        .arg(root)
        .output()
        .expect("failed to run specsync");
    (
        String::from_utf8_lossy(&output.stdout).to_string(),
        String::from_utf8_lossy(&output.stderr).to_string(),
        output.status.code().unwrap_or(-1),
    )
}

/// True when the line contains a digit immediately followed by `%` — a
/// rendered percentage, as opposed to a count or a path.
fn states_a_percentage(line: &str) -> bool {
    line.as_bytes()
        .windows(2)
        .any(|pair| pair[0].is_ascii_digit() && pair[1] == b'%')
}

/// Lines that report a coverage figure. The `--require-coverage` message is
/// excluded: the threshold it echoes is the caller's input, not a measurement.
fn coverage_lines(text: &str) -> Vec<&str> {
    text.lines()
        .filter(|line| {
            let lower = line.to_ascii_lowercase();
            (lower.contains("coverage") || lower.contains("**files:") || lower.contains("**loc:"))
                && !lower.contains("--require-coverage")
        })
        .collect()
}

/// No line reporting coverage may state a percentage when the denominator was
/// zero. Catches `(100%)`, `100% (0/0)`, `(100.0%)` and every other shape a
/// renderer might reach for, without matching counts like `0 failed`.
fn assert_no_percentage(label: &str, text: &str) {
    for line in coverage_lines(text) {
        assert!(
            !states_a_percentage(line),
            "{label} stated a coverage percentage over a tree where nothing was \
             measured:\n  {line}\nfull output:\n{text}"
        );
    }
}

// ─── Zero source files ───────────────────────────────────────────────────

#[test]
fn no_command_or_format_reports_a_file_percentage_over_zero_source_files() {
    let tmp = TempDir::new().unwrap();
    let root = tmp.path();
    zero_source_project(root);

    for command in COMMANDS {
        for format in FORMATS {
            let (stdout, _, _) = run(root, &[command, "--format", format]);
            assert_no_percentage(&format!("`{command} --format {format}`"), &stdout);
        }
    }

    // CSV states its figures as bare numbers, so the `%` scan cannot see them.
    let (stdout, _, _) = run(root, &["report", "--format", "csv"]);
    let summary = stdout
        .lines()
        .find(|line| line.starts_with("SUMMARY,"))
        .expect("report --format csv must emit a SUMMARY row");
    assert_eq!(
        summary, "SUMMARY,,,,0,,0",
        "the CSV summary must leave the coverage field empty, not write a number \
         nobody measured:\n{stdout}"
    );
    let module_row = stdout
        .lines()
        .find(|line| line.starts_with("auth,"))
        .expect("report --format csv must emit the module row");
    assert!(
        module_row.contains("specs/auth/auth.spec.md,active,100,"),
        "the module's own files-exist figure is genuinely measured and must survive:\n{stdout}"
    );
}

#[test]
fn coverage_json_reports_null_not_a_number_over_zero_source_files() {
    let tmp = TempDir::new().unwrap();
    let root = tmp.path();
    zero_source_project(root);

    let (stdout, _, _) = run(root, &["coverage", "--format", "json"]);
    let json: serde_json::Value = serde_json::from_str(&stdout).unwrap();
    assert!(
        json["file_coverage"].is_null(),
        "file_coverage must be null when no file was measured: {json}"
    );
    assert!(
        json["loc_coverage"].is_null(),
        "loc_coverage must be null when no line was measured: {json}"
    );
    assert_eq!(json["files_total"], 0, "{json}");
    assert_eq!(json["files_covered"], 0, "{json}");
}

#[test]
fn report_json_reports_null_overall_coverage_over_zero_source_files() {
    let tmp = TempDir::new().unwrap();
    let root = tmp.path();
    zero_source_project(root);

    let (stdout, _, _) = run(root, &["report", "--format", "json"]);
    let json: serde_json::Value = serde_json::from_str(&stdout).unwrap();
    assert!(
        json["overall_coverage_pct"].is_null(),
        "overall_coverage_pct must be null when no file was measured: {json}"
    );
}

#[test]
fn check_json_reports_null_coverage_percent_for_a_project_with_no_specs() {
    let tmp = TempDir::new().unwrap();
    let root = tmp.path();
    write_config(root, "specs", &["src"]);
    fs::create_dir_all(root.join("src")).unwrap();
    fs::create_dir_all(root.join("specs")).unwrap();

    let (stdout, _, _) = run(root, &["check", "--format", "json"]);
    let json: serde_json::Value = serde_json::from_str(&stdout).unwrap();
    assert_eq!(json["total_source_files"], 0, "{json}");
    assert!(
        json["coverage_percent"].is_null(),
        "coverage_percent must be null when no file was measured: {json}"
    );
}

#[test]
fn text_and_markdown_say_nothing_was_measured_over_zero_source_files() {
    let tmp = TempDir::new().unwrap();
    let root = tmp.path();
    zero_source_project(root);

    // The wording is the same in the terminal renderer and the markdown one:
    // markdown used to print `0/0 (100%)` for the identical report.
    for format in ["text", "markdown", "github"] {
        let (stdout, _, _) = run(root, &["check", "--format", format]);
        assert!(
            stdout.contains("no source files to measure"),
            "`check --format {format}` must state that nothing was measured:\n{stdout}"
        );
        assert!(
            stdout.contains("no source lines to measure"),
            "`check --format {format}` must state that no lines were measured:\n{stdout}"
        );
    }
}

#[test]
fn comment_body_states_nothing_was_measured_over_zero_source_files() {
    let tmp = TempDir::new().unwrap();
    let root = tmp.path();
    zero_source_project(root);

    let (stdout, _, _) = run(root, &["comment"]);
    assert!(
        stdout.contains("| File coverage | no source files to measure (0/0) |"),
        "the PR comment must not name a percentage it does not have:\n{stdout}"
    );
    assert!(
        stdout.contains("| LOC coverage | no source lines to measure (0/0) |"),
        "the PR comment must not name a LOC percentage it does not have:\n{stdout}"
    );
}

#[test]
fn mcp_coverage_surfaces_null_not_a_hundred_over_zero_source_files() {
    let tmp = TempDir::new().unwrap();
    let root = tmp.path();
    zero_source_project(root);

    let responses = mcp_request(
        root,
        &[
            serde_json::json!({
                "jsonrpc": "2.0", "id": 1, "method": "tools/call",
                "params": { "name": "specsync_coverage", "arguments": {} }
            }),
            serde_json::json!({
                "jsonrpc": "2.0", "id": 2, "method": "tools/call",
                "params": { "name": "specsync_check", "arguments": {} }
            }),
            serde_json::json!({
                "jsonrpc": "2.0", "id": 3, "method": "resources/read",
                "params": { "uri": "specsync:///coverage" }
            }),
        ],
    );

    let tool_coverage: serde_json::Value = serde_json::from_str(
        responses[0]["result"]["content"][0]["text"]
            .as_str()
            .unwrap(),
    )
    .unwrap();
    assert!(
        tool_coverage["file_coverage"].is_null(),
        "specsync_coverage must not hand an agent a fabricated 100: {tool_coverage}"
    );
    assert!(
        tool_coverage["loc_coverage"].is_null(),
        "specsync_coverage must not hand an agent a fabricated LOC 100: {tool_coverage}"
    );

    let tool_check: serde_json::Value = serde_json::from_str(
        responses[1]["result"]["content"][0]["text"]
            .as_str()
            .unwrap(),
    )
    .unwrap();
    assert!(
        tool_check["coverage"]["file_percent"].is_null(),
        "specsync_check must not report a coverage percent it did not measure: {tool_check}"
    );
    assert!(
        tool_check["coverage"]["loc_percent"].is_null(),
        "specsync_check must not report a LOC percent it did not measure: {tool_check}"
    );

    let resource: serde_json::Value = serde_json::from_str(
        responses[2]["result"]["contents"][0]["text"]
            .as_str()
            .unwrap(),
    )
    .unwrap();
    assert!(
        resource["file_coverage_percent"].is_null(),
        "the coverage resource must not report a percentage it did not measure: {resource}"
    );
    assert!(
        resource["loc_coverage_percent"].is_null(),
        "the coverage resource must not report a LOC percentage it did not measure: {resource}"
    );
}

// ─── The contradiction the issue names ───────────────────────────────────

#[test]
fn require_coverage_fails_closed_and_no_payload_claims_the_gate_was_met() {
    let tmp = TempDir::new().unwrap();
    let root = tmp.path();
    zero_source_project(root);

    // `compute_exit_code` already refused this tree while the JSON alongside it
    // said `"coverage_percent": 100` — two mechanisms disagreeing about one
    // tree. The gate still fails; now the payload agrees with it.
    for command in ["check", "coverage", "report", "deps"] {
        for format in ["text", "json"] {
            let (stdout, _, code) = run(
                root,
                &[command, "--format", format, "--require-coverage", "80"],
            );
            assert_eq!(
                code, 1,
                "`{command} --format {format} --require-coverage 80` must fail closed over an \
                 unmeasured tree; stdout={stdout}"
            );
            assert_no_percentage(
                &format!("`{command} --format {format} --require-coverage 80`"),
                &stdout,
            );
        }
    }
}

#[test]
fn require_coverage_explains_that_nothing_was_measured() {
    let tmp = TempDir::new().unwrap();
    let root = tmp.path();
    zero_source_project(root);

    specsync()
        .args(["check", "--require-coverage", "80", "--root"])
        .arg(root)
        .assert()
        .failure()
        .stdout(predicate::str::contains("no source files were found"));
}

// ─── Zero lines of code ──────────────────────────────────────────────────

#[test]
fn no_command_or_format_reports_a_loc_percentage_over_zero_lines() {
    let tmp = TempDir::new().unwrap();
    let root = tmp.path();
    zero_loc_project(root);

    // File coverage IS measured here (1/1), so only the LOC half must abstain.
    for command in COMMANDS {
        for format in FORMATS {
            let (stdout, _, _) = run(root, &[command, "--format", format]);
            assert!(
                !stdout.contains("LOC coverage | 100%")
                    && !stdout.contains("**LOC:** 0/0 (100%)")
                    && !stdout.contains("LOC coverage:  0/0 (100%)"),
                "`{command} --format {format}` reported a LOC percentage over zero lines:\n{stdout}"
            );
        }
    }

    let (stdout, _, _) = run(root, &["coverage", "--format", "json"]);
    let json: serde_json::Value = serde_json::from_str(&stdout).unwrap();
    assert!(
        json["loc_coverage"].is_null(),
        "loc_coverage must be null when no line was measured: {json}"
    );
    assert_eq!(
        json["file_coverage"], 100.0,
        "file coverage IS measured here and must still be reported: {json}"
    );
}

// ─── Healthy control: the fix must not silence a real measurement ────────

#[test]
fn healthy_project_still_reports_its_percentages() {
    let tmp = TempDir::new().unwrap();
    let root = tmp.path();
    healthy_project(root);

    let (stdout, _, _) = run(root, &["coverage", "--format", "json"]);
    let json: serde_json::Value = serde_json::from_str(&stdout).unwrap();
    assert_eq!(json["file_coverage"], 50.0, "{json}");
    assert_eq!(json["files_total"], 2, "{json}");
    assert_eq!(json["files_covered"], 1, "{json}");
    assert!(
        json["loc_coverage"].as_f64().is_some_and(|v| v > 0.0),
        "a measured tree must still report LOC coverage: {json}"
    );

    let (stdout, _, _) = run(root, &["check", "--format", "text"]);
    assert!(
        stdout.contains("File coverage: 1/2 (50%)"),
        "a measured tree must still report its percentage:\n{stdout}"
    );

    let (stdout, _, _) = run(root, &["check", "--format", "markdown"]);
    assert!(
        stdout.contains("- **Files:** 1/2 (50%)"),
        "markdown must still report a measured percentage:\n{stdout}"
    );

    let (_, _, code) = run(root, &["check", "--require-coverage", "40"]);
    assert_eq!(code, 0, "a 50% tree must still satisfy a 40% gate");
    let (_, _, code) = run(root, &["check", "--require-coverage", "60"]);
    assert_eq!(code, 1, "a 50% tree must still fail a 60% gate");
}

// ─── Missing files: one denominator, not two ─────────────────────────────

#[test]
fn a_file_a_spec_claims_but_that_is_absent_counts_in_every_denominator() {
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
        valid_spec("auth", &["src/auth/service.ts", "src/auth/ghost.ts"]),
    )
    .unwrap();

    // The gate has always counted the absent file (one real file covered out of
    // two claimed = 50%), while `coverage --json` re-derived its own percentage
    // over the files it could see and reported 100.0. Same tree, two answers.
    let (stdout, _, _) = run(root, &["coverage", "--format", "json"]);
    let json: serde_json::Value = serde_json::from_str(&stdout).unwrap();
    assert_eq!(
        json["file_coverage"], 50.0,
        "the JSON percentage must use the same denominator as the gate: {json}"
    );

    let (_, _, code) = run(root, &["coverage", "--require-coverage", "100"]);
    assert_eq!(code, 1, "a spec claiming an absent file cannot be 100%");
}
