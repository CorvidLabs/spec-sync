//! #576 — the set of findings a run reports must not depend on `--format`.
//!
//! The defect class this campaign keeps shipping: a category is empty for want
//! of INPUT, and the code reads that as want of PROBLEMS. Here the missing
//! input was the renderer itself. `check --format table` and `--format csv`
//! fell into the shared `Text | Table | Csv` arm, which prints only the summary
//! and the coverage line, so a run that exited 1 produced output in which no
//! finding appeared — a CSV consumer saw zero rows and concluded the project
//! was clean while the exit code said the opposite. `coverage --format json`
//! ran the full validation and then dropped its results on the floor
//! (`_all_errors`, `_all_warnings`, `_all_notices`), emitting a spotless
//! `{file_coverage: 100, modules: [], uncovered_files: []}` beside exit 1.
//!
//! Presentation may differ between formats. The SET may not. Every test here
//! extracts the finding identities from each format's own rendering and
//! asserts the sets are equal — a bespoke parser per format, because agreeing
//! on a shared string would only prove the formats share a renderer, not that
//! each one really shows the user the findings.

use crate::helpers::*;
use std::collections::BTreeSet;
use std::fs;
use tempfile::TempDir;

/// Every format the two commands accept.
const FORMATS: [&str; 6] = ["text", "json", "markdown", "github", "table", "csv"];

// ─── Fixtures ────────────────────────────────────────────────────────────

/// Source with two exports; specs document one of them in the broken fixture
/// and both in the clean one.
const CALC_SRC: &str = "export function mul(a: number, b: number): number { return a * b; }\n\
                        export function sub(a: number, b: number): number { return a - b; }\n";

/// A spec documenting `rows` in its Exported Functions table.
fn calc_spec(rows: &str) -> String {
    format!(
        r#"---
module: calc
version: 1
status: active
files:
  - src/calc/calc.ts
db_tables: []
depends_on: []
---

# Calc

## Purpose

Arithmetic helpers used by the billing pipeline.

## Public API

### Exported Functions

| Function | Parameters | Returns | Description |
|----------|-----------|---------|-------------|
{rows}

### Exported Types

| Type | Description |
|------|-------------|

## Invariants

1. Multiplication and subtraction are total over the number domain.

## Behavioral Examples

### Scenario: Multiply two numbers

- **Given** two numbers
- **When** `mul` is called
- **Then** their product is returned

## Error Cases

| Condition | Behavior |
|-----------|----------|
| Non-numeric input | TypeScript rejects it at compile time |

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
| 2026-01-01 | team | Initial version |
"#
    )
}

/// `mul` is documented, `sub` is not — one undocumented-export finding plus the
/// export-ratio finding that accompanies it.
fn broken_project(root: &std::path::Path) {
    write_config(root, "specs", &["src"]);
    fs::create_dir_all(root.join("src/calc")).unwrap();
    fs::write(root.join("src/calc/calc.ts"), CALC_SRC).unwrap();
    fs::create_dir_all(root.join("specs/calc")).unwrap();
    fs::write(
        root.join("specs/calc/calc.spec.md"),
        calc_spec("| `mul` | a, b | number | Multiplies its arguments. |"),
    )
    .unwrap();
}

/// Both exports documented: the healthy control. Every format must agree that
/// there is nothing to report — the fix must not manufacture findings.
fn clean_project(root: &std::path::Path) {
    write_config(root, "specs", &["src"]);
    fs::create_dir_all(root.join("src/calc")).unwrap();
    fs::write(root.join("src/calc/calc.ts"), CALC_SRC).unwrap();
    fs::create_dir_all(root.join("specs/calc")).unwrap();
    fs::write(
        root.join("specs/calc/calc.spec.md"),
        calc_spec(
            "| `mul` | a, b | number | Multiplies its arguments. |\n\
             | `sub` | a, b | number | Subtracts its second argument from its first. |",
        ),
    )
    .unwrap();
}

// ─── Running ─────────────────────────────────────────────────────────────

/// Run one command in one format.
///
/// `check --force` is not incidental: `check` writes a hash cache when it finds
/// no errors, so the FIRST format run would warm it and every later one would
/// skip re-validation. Without it this test would compare formats against a
/// moving input and blame the difference on presentation. (`coverage` has no
/// such flag and consults no cache — it re-validates every run.)
fn run(root: &std::path::Path, command: &str, format: &str, extra: &[&str]) -> (String, i32) {
    let mut cmd = specsync();
    cmd.arg(command).args(["--format", format]);
    if command == "check" {
        cmd.arg("--force");
    }
    let output = cmd
        .args(extra)
        .arg("--root")
        .arg(root)
        .output()
        .expect("failed to run specsync");
    (
        String::from_utf8_lossy(&output.stdout).to_string(),
        output.status.code().unwrap_or(-1),
    )
}

// ─── Identity extraction, one parser per format ──────────────────────────

/// A finding identity: `"{spec}: {message}"`, the form every machine-readable
/// payload already uses.
type Identities = BTreeSet<String>;

/// Rebuild the identity from a format that stores the two parts separately.
///
/// A project-scoped finding has no spec, and its identity is the bare message —
/// not `": {message}"`, which would make an unattributed finding compare
/// unequal to itself across formats.
fn identity(spec: &str, message: &str) -> String {
    if spec.is_empty() {
        message.to_string()
    } else {
        format!("{spec}: {message}")
    }
}

/// `check --format json` / `coverage --format json`: the arrays are the set.
fn from_json(stdout: &str) -> Identities {
    let json: serde_json::Value =
        serde_json::from_str(stdout).unwrap_or_else(|e| panic!("not JSON ({e}):\n{stdout}"));
    ["errors", "warnings"]
        .iter()
        .flat_map(|key| {
            json[key]
                .as_array()
                .unwrap_or_else(|| panic!("`{key}` must be an array:\n{json:#}"))
                .iter()
                .map(|v| v.as_str().unwrap_or_default().to_string())
                .collect::<Vec<_>>()
        })
        .collect()
}

/// The per-spec terminal renderer: an unindented spec path opens a block, and
/// each `⚠`/`✗` line at two-space indent inside it is one finding. Also used
/// for every `coverage` format except json, which all render as text today.
fn from_text(stdout: &str) -> Identities {
    let mut set = Identities::new();
    let mut spec = String::new();
    for line in stdout.lines() {
        if !line.starts_with(' ') && line.trim_end().ends_with(".spec.md") {
            spec = line.trim().to_string();
            continue;
        }
        if let Some(rest) = line
            .strip_prefix("  ⚠ ")
            .or_else(|| line.strip_prefix("  ✗ "))
        {
            set.insert(identity(&spec, rest));
        }
    }
    set
}

/// `--format markdown`: flat `- {spec}: {message}` bullets under `### Errors`
/// and `### Warnings`.
fn from_markdown(stdout: &str) -> Identities {
    let mut set = Identities::new();
    let mut in_findings = false;
    for line in stdout.lines() {
        if let Some(heading) = line.strip_prefix("### ") {
            in_findings = matches!(heading.trim(), "Errors" | "Warnings");
            continue;
        }
        if in_findings && let Some(rest) = line.strip_prefix("- ") {
            set.insert(rest.to_string());
        }
    }
    set
}

/// `--format github`: `### Errors` / `### Warnings` sections, each spec in a
/// `**`backticked path`**` subheader with bare `- {message}` bullets under it.
/// The `### Action Items` checklist repeats the same messages in another shape
/// and is deliberately not a second source of identities.
fn from_github(stdout: &str) -> Identities {
    let mut set = Identities::new();
    let mut in_findings = false;
    let mut spec = String::new();
    for line in stdout.lines() {
        if let Some(heading) = line.strip_prefix("### ") {
            in_findings = matches!(heading.trim(), "Errors" | "Warnings");
            continue;
        }
        if !in_findings {
            continue;
        }
        if let Some(inner) = line
            .strip_prefix("**`")
            .and_then(|rest| rest.strip_suffix("`**"))
        {
            spec = inner.to_string();
            continue;
        }
        if let Some(rest) = line.strip_prefix("- ") {
            set.insert(identity(&spec, rest));
        }
    }
    set
}

/// `--format table`: `SEVERITY  SPEC  MESSAGE` in fixed-width columns.
///
/// Parsed by the widths the rule row declares rather than by splitting on runs
/// of spaces — an empty SPEC cell (a project-scoped finding) is nothing but
/// padding, and a space-splitting parser silently shifts the message into it.
fn from_table(stdout: &str) -> Identities {
    let mut lines = stdout.lines();
    let header = lines
        .next()
        .expect("table output must start with a header row");
    assert!(
        header.starts_with("SEVERITY") && header.contains("SPEC") && header.contains("MESSAGE"),
        "table output must open with the findings header, got: {header:?}\n{stdout}"
    );
    let rule = lines.next().expect("table output must have a rule row");
    let widths: Vec<usize> = rule
        .split("  ")
        .map(|dashes| dashes.chars().count())
        .collect();
    assert_eq!(
        widths.len(),
        3,
        "the rule row must declare 3 columns: {rule:?}"
    );

    lines
        .take_while(|line| !line.trim().is_empty())
        .map(|line| {
            let chars: Vec<char> = line.chars().collect();
            let take = |start: usize, width: usize| -> String {
                chars
                    .iter()
                    .skip(start)
                    .take(width)
                    .collect::<String>()
                    .trim()
                    .to_string()
            };
            let spec_start = widths[0] + 2;
            let message_start = spec_start + widths[1] + 2;
            let spec = take(spec_start, widths[1]);
            let message: String = chars.iter().skip(message_start).collect::<String>();
            identity(&spec, message.trim())
        })
        .collect()
}

/// `--format csv`: `severity,spec,message` with RFC4180 quoting.
fn from_csv(stdout: &str) -> Identities {
    let mut lines = stdout.lines();
    assert_eq!(
        lines.next(),
        Some("severity,spec,message"),
        "CSV output must open with the stable header:\n{stdout}"
    );
    lines
        .filter(|line| !line.trim().is_empty())
        .map(|line| {
            let fields = parse_csv_row(line);
            assert_eq!(
                fields.len(),
                3,
                "every CSV row must have exactly three columns, got {fields:?}"
            );
            identity(&fields[1], &fields[2])
        })
        .collect()
}

/// Minimal RFC4180 row parser — enough to prove the rows really are CSV and
/// that an embedded comma does not become a fourth column.
fn parse_csv_row(line: &str) -> Vec<String> {
    let mut fields = Vec::new();
    let mut current = String::new();
    let mut quoted = false;
    let mut chars = line.chars().peekable();
    while let Some(c) = chars.next() {
        match c {
            '"' if quoted && chars.peek() == Some(&'"') => {
                current.push('"');
                chars.next();
            }
            '"' => quoted = !quoted,
            ',' if !quoted => fields.push(std::mem::take(&mut current)),
            _ => current.push(c),
        }
    }
    fields.push(current);
    fields
}

/// Dispatch to the parser that matches how this command renders this format.
fn identities(command: &str, format: &str, stdout: &str) -> Identities {
    match (command, format) {
        (_, "json") => from_json(stdout),
        // `coverage` renders every non-json format through the text renderer
        // today (its format flag is a no-op for markdown/github/table/csv —
        // a missing feature, tracked separately, NOT this defect). Parsing it
        // as text is therefore what its output actually is.
        ("coverage", _) => from_text(stdout),
        (_, "text") => from_text(stdout),
        (_, "markdown") => from_markdown(stdout),
        (_, "github") => from_github(stdout),
        (_, "table") => from_table(stdout),
        (_, "csv") => from_csv(stdout),
        _ => unreachable!("unhandled format {format}"),
    }
}

// ─── The regression ──────────────────────────────────────────────────────

#[test]
fn every_format_of_check_reports_the_same_finding_set() {
    let tmp = TempDir::new().unwrap();
    let root = tmp.path();
    broken_project(root);

    let mut sets: Vec<(&str, Identities)> = Vec::new();
    for format in FORMATS {
        let (stdout, _) = run(root, "check", format, &[]);
        sets.push((format, identities("check", format, &stdout)));
    }

    let (reference_format, reference) = &sets[0];
    assert!(
        reference
            .iter()
            .any(|f| f.contains("Undocumented export 'sub'")),
        "the fixture must actually be broken; `check --format {reference_format}` found {reference:#?}"
    );
    for (format, set) in &sets[1..] {
        assert_eq!(
            set, reference,
            "`check --format {format}` reports a different finding set than \
             `--format {reference_format}`.\n  {format}: {set:#?}\n  {reference_format}: {reference:#?}"
        );
    }
}

/// What this asserts, precisely: given the same validation INPUTS, the finding
/// set does not depend on the command or the format.
///
/// It deliberately does not assert that `check` and `coverage` always agree.
/// They do not, and did not before this change: `cmd_coverage` validates with
/// `IgnoreRules::default()` while `cmd_check` uses `IgnoreRules::load(root)`,
/// so on a project with a `.specsyncignore` the two produce different warning
/// lists (and different `--strict` exit codes). That is a divergence in what is
/// validated, not in how it is rendered — a separate question from this one,
/// and the fixture here has no ignore file so it cannot mask a rendering bug.
#[test]
fn every_format_of_coverage_reports_the_same_finding_set_as_check() {
    let tmp = TempDir::new().unwrap();
    let root = tmp.path();
    broken_project(root);

    let (check_stdout, _) = run(root, "check", "json", &[]);
    let reference = from_json(&check_stdout);
    assert!(
        !reference.is_empty(),
        "the fixture must actually be broken:\n{check_stdout}"
    );

    for format in FORMATS {
        let (stdout, _) = run(root, "coverage", format, &[]);
        assert_eq!(
            identities("coverage", format, &stdout),
            reference,
            "`coverage --format {format}` reports a different finding set than `check`:\n{stdout}"
        );
    }
}

#[test]
fn coverage_json_carries_the_findings_it_exits_on() {
    let tmp = TempDir::new().unwrap();
    let root = tmp.path();
    broken_project(root);

    // The exact contradiction the issue names: exit 1 beside a payload in which
    // nothing is wrong. `--strict` makes the warnings gate.
    let (stdout, code) = run(root, "coverage", "json", &["--strict"]);
    assert_eq!(code, 1, "the gate must still fail:\n{stdout}");

    let json: serde_json::Value = serde_json::from_str(&stdout).unwrap();
    assert_eq!(
        json["passed"], false,
        "a payload printed beside exit 1 must not say the run passed:\n{json:#}"
    );
    assert!(
        !json["warnings"].as_array().unwrap().is_empty(),
        "the findings the exit code is based on must be IN the payload:\n{json:#}"
    );
    assert_eq!(
        json["total_warnings"],
        json["warnings"].as_array().unwrap().len(),
        "the count and the array must agree:\n{json:#}"
    );
}

#[test]
fn table_and_csv_name_the_findings_they_exit_on() {
    let tmp = TempDir::new().unwrap();
    let root = tmp.path();
    broken_project(root);

    for format in ["table", "csv"] {
        let (stdout, code) = run(root, "check", format, &["--strict"]);
        assert_eq!(code, 1, "`check --format {format} --strict` must fail");
        assert!(
            stdout.contains("Undocumented export 'sub'"),
            "`check --format {format}` exited 1 without naming a single finding — \
             a consumer parsing this sees no problems while the exit code disagrees:\n{stdout}"
        );
    }
}

#[test]
fn csv_stdout_is_csv_and_nothing_else() {
    let tmp = TempDir::new().unwrap();
    let root = tmp.path();
    broken_project(root);

    // Prose on stdout is what makes a CSV unparseable, and the `--strict` gate
    // message used to be printed there with `println!`.
    let (stdout, _) = run(root, "check", "csv", &["--strict"]);
    for line in stdout.lines().filter(|l| !l.trim().is_empty()) {
        assert_eq!(
            parse_csv_row(line).len(),
            3,
            "every stdout line must be a three-column CSV row, got {line:?}:\n{stdout}"
        );
    }
    assert!(
        !stdout.contains("--strict mode"),
        "the gate message belongs on stderr, not in the CSV:\n{stdout}"
    );
    assert!(
        !stdout.contains("File coverage:"),
        "the coverage line belongs on stderr, not in the CSV:\n{stdout}"
    );
}

#[test]
fn a_finding_containing_a_comma_stays_one_csv_column() {
    let tmp = TempDir::new().unwrap();
    let root = tmp.path();
    broken_project(root);
    // A spec naming a module that does not exist produces a finding whose text
    // carries punctuation; the point is that whatever it contains, the row
    // still parses as three columns and the message survives intact.
    let spec = root.join("specs/calc/calc.spec.md");
    let content = fs::read_to_string(&spec).unwrap();
    fs::write(
        &spec,
        content.replace(
            "depends_on: []",
            "depends_on:\n  - specs/nope, really/x.spec.md",
        ),
    )
    .unwrap();

    let (stdout, _) = run(root, "check", "csv", &[]);
    let rows: Vec<Vec<String>> = stdout
        .lines()
        .skip(1)
        .filter(|l| !l.trim().is_empty())
        .map(parse_csv_row)
        .collect();
    assert!(
        rows.iter().all(|r| r.len() == 3),
        "a comma inside a finding must not become a new column:\n{stdout}"
    );
    assert!(
        rows.iter().any(|r| r[2].contains("nope, really")),
        "the comma-bearing message must survive quoting intact:\n{stdout}"
    );
}

#[test]
fn staleness_findings_reach_the_tabular_formats_too() {
    // Found by sweeping the fix's own blast radius: staleness findings live in
    // `stale_entries`, not `all_warnings`, and they are counted in
    // `effective_warnings` — so they drive the exit code. Rendering only
    // `all_warnings` would have left `check --format csv --stale` exiting 1
    // over git drift with no row saying why: the same defect, one variable over.
    let tmp = TempDir::new().unwrap();
    let root = tmp.path();
    let git = |args: &[&str]| {
        std::process::Command::new("git")
            .args(args)
            .current_dir(root)
            .output()
            .unwrap();
    };
    git(&["init"]);
    git(&["config", "user.email", "test@test.com"]);
    git(&["config", "user.name", "Test"]);

    clean_project(root);
    git(&["add", "."]);
    git(&["commit", "-m", "initial"]);

    // Move the source well ahead of the spec's last commit.
    for n in 0..4 {
        fs::write(
            root.join("src/calc/calc.ts"),
            format!("{CALC_SRC}// revision {n}\n"),
        )
        .unwrap();
        git(&["add", "."]);
        git(&["commit", "-m", &format!("edit {n}")]);
    }

    // All four non-text formats, not just the two that were fixed first.
    // Restricting this loop to table and csv is exactly why markdown and
    // github kept exiting 1 while naming no staleness finding.
    for format in ["table", "csv", "markdown", "github"] {
        let (stdout, _) = run(root, "check", format, &["--stale", "2"]);
        assert!(
            stdout.contains("commits behind source files"),
            "`check --format {format} --stale` counted a staleness warning in its \
             summary but named it in no row:\n{stdout}"
        );
    }

    // And the row count still matches the number the summary states.
    let (stdout, _) = run(root, "check", "csv", &["--stale", "2"]);
    let rows = stdout
        .lines()
        .skip(1)
        .filter(|l| !l.trim().is_empty())
        .count();
    assert_eq!(
        rows, 1,
        "one staleness finding must produce exactly one CSV row:\n{stdout}"
    );
}

/// Not every finding belongs to a spec. A dead `.specsyncignore` rule is a
/// project-scoped one, and the tabular renderers must show it with an EMPTY
/// spec cell rather than drop it for having nothing to key on — dropping it
/// would be this very defect class one layer down.
///
/// Compared against `--format json` rather than against every format: json,
/// table and csv are the three collect-mode renderers and must agree exactly.
/// `--format text` is deliberately excluded, and not because it hides
/// anything — it prints this warning twice, as does json. But collect mode
/// pushes the ignore-rule warning under BOTH spellings (`.specsyncignore:
/// <msg>` from the ignore-rules seed at `commands/mod.rs:615`, and a bare
/// `<msg>` from the loop at `commands/mod.rs:1072`), while text prints one
/// string twice. That double-count is pre-existing — the base binary shows the
/// same two-vs-two — and it is a duplication defect, not an absence one, so it
/// is reported rather than fixed here.
#[test]
fn a_project_scoped_finding_survives_with_an_empty_spec_column() {
    let tmp = TempDir::new().unwrap();
    let root = tmp.path();
    broken_project(root);
    fs::write(root.join(".specsyncignore"), "this-is-not-a-category\n").unwrap();

    let reference = from_json(&run(root, "check", "json", &[]).0);
    assert!(
        reference
            .iter()
            .any(|f| f.contains("matches no warning category")),
        "the fixture must produce a project-scoped finding: {reference:#?}"
    );

    for format in ["table", "csv"] {
        let (stdout, _) = run(root, "check", format, &[]);
        assert_eq!(
            identities("check", format, &stdout),
            reference,
            "`check --format {format}` lost or mangled a project-scoped finding:\n{stdout}"
        );
        assert!(
            stdout.contains("matches no warning category"),
            "a finding with no spec to key on must still be rendered:\n{stdout}"
        );
    }
}

// ─── Healthy control: both directions ────────────────────────────────────

#[test]
fn a_clean_project_is_all_clear_in_every_format() {
    let tmp = TempDir::new().unwrap();
    let root = tmp.path();
    clean_project(root);

    for command in ["check", "coverage"] {
        for format in FORMATS {
            let (stdout, code) = run(root, command, format, &["--strict"]);
            assert_eq!(
                code, 0,
                "`{command} --format {format} --strict` must pass on a clean project:\n{stdout}"
            );
            assert!(
                identities(command, format, &stdout).is_empty(),
                "`{command} --format {format}` invented findings on a clean project:\n{stdout}"
            );
        }
    }
}

#[test]
fn a_clean_project_still_reports_its_measured_coverage() {
    let tmp = TempDir::new().unwrap();
    let root = tmp.path();
    clean_project(root);

    // The fix must not silence a real measurement — the failure mode #582 was
    // fixed for.
    let (stdout, _) = run(root, "coverage", "json", &[]);
    let json: serde_json::Value = serde_json::from_str(&stdout).unwrap();
    assert_eq!(json["passed"], true, "{json:#}");
    assert_eq!(json["file_coverage"], 100.0, "{json:#}");
    assert_eq!(json["files_total"], 1, "{json:#}");
    assert_eq!(json["specs_checked"], 1, "{json:#}");
    assert_eq!(json["total_errors"], 0, "{json:#}");
    assert_eq!(json["total_warnings"], 0, "{json:#}");

    // An empty findings CSV is a header and zero rows, not empty output: a
    // parser must be able to tell "checked, nothing found" from "never ran".
    let (stdout, _) = run(root, "check", "csv", &[]);
    assert_eq!(
        stdout.trim(),
        "severity,spec,message",
        "a clean run must still emit a well-formed empty CSV:\n{stdout}"
    );
}

// ─── The three coverage payloads are one payload ─────────────────────────

#[test]
fn the_cli_and_both_mcp_coverage_payloads_agree() {
    let tmp = TempDir::new().unwrap();
    let root = tmp.path();
    broken_project(root);

    let responses = mcp_request(
        root,
        &[
            serde_json::json!({
                "jsonrpc": "2.0", "id": 1, "method": "tools/call",
                "params": { "name": "specsync_coverage", "arguments": {} }
            }),
            serde_json::json!({
                "jsonrpc": "2.0", "id": 2, "method": "resources/read",
                "params": { "uri": "specsync:///coverage" }
            }),
        ],
    );
    let tool: serde_json::Value = serde_json::from_str(
        responses[0]["result"]["content"][0]["text"]
            .as_str()
            .unwrap(),
    )
    .unwrap();
    let resource: serde_json::Value = serde_json::from_str(
        responses[1]["result"]["contents"][0]["text"]
            .as_str()
            .unwrap(),
    )
    .unwrap();

    // The MCP resource used to be the site everyone forgot: no findings, no
    // `passed`, its own zero-denominator percentage. One constructor now, so
    // the two payloads are byte-identical rather than merely similar.
    assert_eq!(
        tool, resource,
        "the `specsync_coverage` tool and the `specsync:///coverage` resource must \
         be the same payload"
    );
    assert!(
        !tool["warnings"].as_array().unwrap().is_empty(),
        "an agent reading the coverage surface must see the findings:\n{tool:#}"
    );

    // …and the same shape the CLI emits, with the same finding identities.
    let (stdout, _) = run(root, "coverage", "json", &[]);
    let cli: serde_json::Value = serde_json::from_str(&stdout).unwrap();
    let keys = |v: &serde_json::Value| -> BTreeSet<String> {
        v.as_object().unwrap().keys().cloned().collect()
    };
    assert_eq!(
        keys(&cli),
        keys(&tool),
        "the CLI and MCP coverage payloads must have the same fields"
    );
    assert_eq!(
        from_json(&stdout),
        ["errors", "warnings"]
            .iter()
            .flat_map(|k| tool[k]
                .as_array()
                .unwrap()
                .iter()
                .map(|v| v.as_str().unwrap().to_string())
                .collect::<Vec<_>>())
            .collect::<Identities>(),
        "the CLI and MCP coverage surfaces must report the same findings"
    );
}

#[test]
fn every_coverage_payload_reports_absence_as_null_not_a_hundred() {
    let tmp = TempDir::new().unwrap();
    let root = tmp.path();
    // Zero source files: nothing was measured. Routing all three payloads
    // through one constructor must not resurrect the fabricated 100 that #582
    // removed — the constructor is now the single place that could.
    write_config(root, "specs", &["src"]);
    fs::create_dir_all(root.join("src/calc")).unwrap();
    fs::write(root.join("src/calc/NOTES.md"), "notes\n").unwrap();
    fs::create_dir_all(root.join("specs/calc")).unwrap();
    fs::write(
        root.join("specs/calc/calc.spec.md"),
        valid_spec("calc", &["src/calc/NOTES.md"]),
    )
    .unwrap();

    let (stdout, _) = run(root, "coverage", "json", &[]);
    let cli: serde_json::Value = serde_json::from_str(&stdout).unwrap();
    for key in [
        "file_coverage",
        "file_coverage_percent",
        "loc_coverage",
        "loc_coverage_percent",
    ] {
        assert!(
            cli[key].is_null(),
            "`{key}` must be null when nothing was measured:\n{cli:#}"
        );
    }
}
