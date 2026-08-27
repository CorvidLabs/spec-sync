use colored::Colorize;
use std::io::Write;

use crate::types;

fn write_summary(
    out: &mut dyn Write,
    total: usize,
    passed: usize,
    warnings: usize,
    _errors: usize,
) {
    // saturating_sub guards against an underflow panic if `passed` is ever
    // reported higher than `total`.
    let failed = total.saturating_sub(passed);
    let _ = writeln!(
        out,
        "\n{total} specs checked: {} passed, {} warning(s), {} failed",
        passed.to_string().green(),
        warnings.to_string().yellow(),
        if failed > 0 {
            failed.to_string().red().to_string()
        } else {
            "0".to_string()
        }
    );
}

pub fn print_summary(total: usize, passed: usize, warnings: usize, errors: usize) {
    write_summary(
        &mut std::io::stdout().lock(),
        total,
        passed,
        warnings,
        errors,
    );
}

/// The same summary on stderr, for formats whose stdout is a machine protocol
/// (CSV). Deliberately the same renderer rather than a second one: the summary
/// a CSV run shows a human must not be free to drift from the one every other
/// format shows.
pub fn eprint_summary(total: usize, passed: usize, warnings: usize, errors: usize) {
    write_summary(
        &mut std::io::stderr().lock(),
        total,
        passed,
        warnings,
        errors,
    );
}

/// Render an optional coverage percentage into a JSON value: the percentage
/// rounded to hundredths, or `null` when there was nothing to measure.
///
/// Machine consumers are the ones a fabricated `100.0` hurts most — a badge or
/// a dashboard has no way to tell it apart from a genuinely covered project.
/// `null` is already how these payloads report an inconclusive discovery, so
/// "not measured" now has one representation across all of them (#582).
pub fn percent_json(percent: Option<f64>) -> serde_json::Value {
    match percent {
        Some(value) => serde_json::json!((value * 100.0).round() / 100.0),
        None => serde_json::Value::Null,
    }
}

/// Quote a CSV field when it contains a delimiter, a quote, or a newline.
///
/// One implementation for every CSV renderer in the codebase. A finding
/// message routinely contains a comma (`Export 'a, b' ...`) and a spec path
/// never does, so a renderer that skips this turns one finding into two
/// columns and shifts every field after it.
pub fn csv_field(value: &str) -> String {
    if value.contains(',') || value.contains('"') || value.contains('\n') || value.contains('\r') {
        format!("\"{}\"", value.replace('"', "\"\""))
    } else {
        value.to_string()
    }
}

/// One validation finding, split into the columns the tabular renderers need.
///
/// Findings are collected as `"{spec}: {message}"` strings (see
/// `ValidationErrors::push_for_spec`); the tabular formats need those two parts
/// as separate columns, and every format must agree on what the parts are.
pub struct Finding<'a> {
    pub severity: &'a str,
    pub spec: &'a str,
    pub message: &'a str,
}

/// Split one rendered finding into `(spec, message)`.
///
/// Project-scoped findings (schema failures, `.specsyncignore` problems) are
/// pushed without a spec prefix; they get an empty spec column rather than
/// being dropped — a finding no format can attribute is still a finding.
fn split_finding<'a>(severity: &'a str, rendered: &'a str) -> Finding<'a> {
    match rendered.split_once(": ") {
        Some((spec, message)) if spec.contains('.') && !spec.contains(' ') => Finding {
            severity,
            spec,
            message,
        },
        _ => Finding {
            severity,
            spec: "",
            message: rendered,
        },
    }
}

/// The full finding set in a stable order: errors, then warnings, then notices.
///
/// Every format renders THIS list. The set a consumer sees must not depend on
/// which `--format` they asked for — presentation may differ, the set may not.
pub fn findings<'a>(
    errors: &'a [String],
    warnings: &'a [String],
    notices: &'a [String],
) -> Vec<Finding<'a>> {
    let mut out = Vec::with_capacity(errors.len() + warnings.len() + notices.len());
    out.extend(errors.iter().map(|e| split_finding("error", e)));
    out.extend(warnings.iter().map(|w| split_finding("warning", w)));
    out.extend(notices.iter().map(|n| split_finding("notice", n)));
    out
}

/// Column header for the findings CSV. Stable: consumers key off these names.
pub const FINDING_CSV_HEADER: &str = "severity,spec,message";

/// Render the findings as CSV — one row per finding, stable columns.
///
/// The header is printed even when there are no findings, so an empty result
/// is a well-formed CSV with zero rows rather than empty output. The two are
/// not the same thing to a parser, and "no output" is what let a consumer
/// conclude there were no problems from a run that exited 1.
pub fn print_findings_csv(findings: &[Finding<'_>]) {
    let out = &mut std::io::stdout().lock();
    let _ = writeln!(out, "{FINDING_CSV_HEADER}");
    for finding in findings {
        let _ = writeln!(
            out,
            "{},{},{}",
            csv_field(finding.severity),
            csv_field(finding.spec),
            csv_field(finding.message)
        );
    }
}

/// Render the findings as an aligned ASCII table.
///
/// Prints the header row even when empty, for the same reason the CSV does:
/// "the renderer ran and found nothing" must be distinguishable from "the
/// renderer never ran".
pub fn print_findings_table(findings: &[Finding<'_>]) {
    const SEVERITY_HEADER: &str = "SEVERITY";
    const SPEC_HEADER: &str = "SPEC";
    const MESSAGE_HEADER: &str = "MESSAGE";

    let severity_width = findings
        .iter()
        .map(|f| f.severity.chars().count())
        .chain(std::iter::once(SEVERITY_HEADER.len()))
        .max()
        .unwrap_or(SEVERITY_HEADER.len());
    let spec_width = findings
        .iter()
        .map(|f| f.spec.chars().count())
        .chain(std::iter::once(SPEC_HEADER.len()))
        .max()
        .unwrap_or(SPEC_HEADER.len());

    let out = &mut std::io::stdout().lock();
    let _ = writeln!(
        out,
        "{:<severity_width$}  {:<spec_width$}  {}",
        SEVERITY_HEADER, SPEC_HEADER, MESSAGE_HEADER
    );
    let _ = writeln!(
        out,
        "{}  {}  {}",
        "-".repeat(severity_width),
        "-".repeat(spec_width),
        "-".repeat(MESSAGE_HEADER.len())
    );
    for finding in findings {
        let _ = writeln!(
            out,
            "{:<severity_width$}  {:<spec_width$}  {}",
            finding.severity, finding.spec, finding.message
        );
    }
}

/// The validation findings that accompany a coverage payload.
///
/// `passed` is the GATE VERDICT — the same boolean the process exit code
/// carries (`exit_code == 0`), honouring `--strict`, `--require-coverage` and
/// the configured enforcement mode. That is the one semantics under which a
/// payload can never contradict the exit code beside it, which is the whole
/// defect: `coverage --format json` exited 1 while handing back a payload with
/// nothing wrong in it.
///
/// Because `passed` answers a POLICY question, it is never the only signal
/// here: `total_errors`/`total_warnings` and the finding arrays answer the
/// factual one. A consumer that wants "is this tree clean" reads the counts;
/// one that wants "did the gate pass" reads `passed`. Neither has to infer the
/// other from silence.
pub struct CoverageFindings {
    pub passed: bool,
    pub specs_checked: usize,
    pub specs_passed: usize,
    pub errors: Vec<String>,
    pub warnings: Vec<String>,
    pub notices: Vec<String>,
}

/// THE constructor for every coverage payload — CLI `coverage --format json`,
/// the `specsync_coverage` MCP tool, and the `specsync:///coverage` MCP
/// resource.
///
/// Those three used to hand-build their own JSON. They disagreed about the
/// percentage (one re-derived it with a 100.0 fallback), about the key names,
/// and about whether findings existed at all: all three shipped a coverage
/// report with no `errors`, no `warnings` and no `passed`, so an agent or a
/// dashboard reading one saw a clean project on a tree that `check` failed.
/// Three hand-built payloads is the shape that rots; there is now one.
///
/// Both historical key spellings are emitted (`file_coverage` /
/// `file_coverage_percent`, `modules` / `uncovered_modules`) so no existing
/// consumer breaks, and so all three payloads carry one field set. The tests
/// assert that the two MCP surfaces are byte-identical (same inputs, same
/// constructor) and that the CLI payload has the same keys and the same finding
/// identities. The CLI's finding list can still differ in CONTENT: it applies
/// `.specsyncignore` suppression and the draft-spec warnings, which the MCP
/// collector does not — a pre-existing difference between the two validation
/// paths, not between the payloads.
pub fn coverage_json(
    coverage: &types::CoverageReport,
    findings: &CoverageFindings,
) -> serde_json::Value {
    let file_coverage = percent_json(coverage.file_coverage());
    let loc_coverage = percent_json(coverage.loc_coverage());

    let modules: Vec<serde_json::Value> = coverage
        .unspecced_modules
        .iter()
        .map(|m| serde_json::json!({ "name": m, "has_spec": false }))
        .collect();

    let uncovered_files: Vec<serde_json::Value> = coverage
        .unspecced_file_loc
        .iter()
        .map(|(f, loc)| serde_json::json!({ "file": f, "loc": loc }))
        .collect();

    serde_json::json!({
        // The gate verdict, identical to the process exit code beside it.
        "passed": findings.passed,
        "specs_checked": findings.specs_checked,
        "specs_passed": findings.specs_passed,
        // The factual verdict, independent of enforcement policy.
        "total_errors": findings.errors.len(),
        "total_warnings": findings.warnings.len(),
        "errors": findings.errors,
        "warnings": findings.warnings,
        "notices": findings.notices,
        "file_coverage": file_coverage,
        "file_coverage_percent": file_coverage,
        "files_covered": coverage.specced_file_count,
        "files_total": coverage.measured_file_total(),
        "loc_coverage": loc_coverage,
        "loc_coverage_percent": loc_coverage,
        "loc_covered": coverage.specced_loc,
        "loc_total": coverage.total_loc,
        "modules": modules,
        "uncovered_modules": modules,
        "uncovered_files": uncovered_files,
        // What shaped the denominator. The percentages above are measured over
        // whatever was left after these were excluded or counted (#546, #582),
        // so they travel with the numbers rather than in a section a consumer
        // may never read.
        "missing_files": coverage.missing_files,
        "skipped_links": coverage.skipped_links,
        "manifest_notices": coverage.manifest_notices,
    })
}

/// Wording used everywhere a file-coverage percentage cannot be stated.
pub const NO_FILES_MEASURED: &str = "no source files to measure";

/// Wording used everywhere a LOC-coverage percentage cannot be stated.
pub const NO_LINES_MEASURED: &str = "no source lines to measure";

fn colored_percent(pct: usize) -> String {
    let text = format!("{pct}%");
    if pct == 100 {
        text.green().to_string()
    } else if pct >= 80 {
        text.yellow().to_string()
    } else {
        text.red().to_string()
    }
}

fn write_coverage_line(out: &mut dyn Write, coverage: &types::CoverageReport) {
    // A zero denominator is not 100% — it is nothing measured. Reporting it as
    // 100% put the display in direct contradiction with the gate: the same run
    // exits 1 from `--require-coverage`, which already refuses this as a
    // vacuous pass, while printing a green `100%` (#562). The number is what
    // ends up on badges and dashboards, so it is the half that must not lie.
    //
    // This used to be the only renderer that got it right, and it got it right
    // by ignoring `coverage_percent` and re-deriving from the counts — which is
    // why eight other sites kept printing 100%. Both halves now come from
    // `CoverageReport`, which has no percentage to report when nothing was
    // measured (#582).
    match coverage.file_coverage_percent() {
        Some(pct) => {
            let _ = writeln!(
                out,
                "File coverage: {}/{} ({})",
                coverage.specced_file_count,
                coverage.measured_file_total(),
                colored_percent(pct)
            );
        }
        None => {
            let _ = writeln!(out, "File coverage: 0/0 ({NO_FILES_MEASURED})");
        }
    }
    match coverage.loc_coverage_percent() {
        Some(pct) => {
            let _ = writeln!(
                out,
                "LOC coverage:  {}/{} ({})",
                coverage.specced_loc,
                coverage.total_loc,
                colored_percent(pct)
            );
        }
        None => {
            let _ = writeln!(out, "LOC coverage:  0/0 ({NO_LINES_MEASURED})");
        }
    }
    write_missing_files_note(out, coverage);
    write_skipped_links(out, coverage);
    write_manifest_notices(out, coverage);
}

pub fn print_coverage_line(coverage: &types::CoverageReport) {
    write_coverage_line(&mut std::io::stdout().lock(), coverage);
}

/// The same coverage figures on stderr, for formats whose stdout is a machine
/// protocol (CSV). One renderer, two destinations — a second hand-rolled
/// coverage line is exactly how `csv: file coverage 100% (0/0)` once appeared
/// next to `table: 0/0 (no source files to measure)` on the same tree.
pub fn eprint_coverage_line(coverage: &types::CoverageReport) {
    write_coverage_line(&mut std::io::stderr().lock(), coverage);
}

/// Report files a spec's `files:` list names that are not on disk, immediately
/// after the coverage figures.
///
/// They are part of the denominator — an absent file can never be covered, so
/// excluding it would let `--require-coverage 100` pass over a spec that points
/// at nothing. But that makes the total larger than the number of files on
/// disk, so the total can only be read honestly next to what inflated it.
/// Printed for the same reason as the skipped links: the number is never shown
/// without what shaped it.
fn write_missing_files_note(out: &mut dyn Write, coverage: &types::CoverageReport) {
    if coverage.missing_files.is_empty() {
        return;
    }
    let _ = writeln!(
        out,
        "{} {} file(s) referenced by specs but missing on disk are counted in the total (they can never be covered)",
        "⚠".yellow(),
        coverage.missing_files.len()
    );
}

/// How many skipped links are named before the rest are summarized.
const SKIPPED_LINK_DISPLAY_LIMIT: usize = 5;

/// Report symlinked entries discovery skipped (#546), immediately after the
/// coverage figures.
///
/// Deliberately printed here rather than with the other findings: skipping a
/// link shrinks the denominator, so these percentages can only be read honestly
/// next to what was excluded from them. A repo that symlinks a vendored tree
/// would otherwise see its coverage *rise* because measurement stopped.
fn write_skipped_links(out: &mut dyn Write, coverage: &types::CoverageReport) {
    if coverage.skipped_links.is_empty() {
        return;
    }
    let shown: Vec<&str> = coverage
        .skipped_links
        .iter()
        .take(SKIPPED_LINK_DISPLAY_LIMIT)
        .map(String::as_str)
        .collect();
    let remaining = coverage.skipped_links.len().saturating_sub(shown.len());
    let listed = shown.join(", ");
    let tail = if remaining > 0 {
        format!("{listed}, and {remaining} more")
    } else {
        listed
    };
    let _ = writeln!(
        out,
        "{} {} symlinked path(s) excluded from coverage (not traversed): {tail}",
        "⚠".yellow(),
        coverage.skipped_links.len()
    );
}

/// Report manifests that could not be parsed and were degraded rather than
/// propagated because the project stated its own `source_dirs` (#723).
///
/// Printed here for the same reason as the skipped links: the manifest also
/// declares modules, so a degraded run names FEWER modules without specs than
/// the tree has. That is a report improved by a measurement that stopped, and
/// it cannot be read honestly apart from the figures it shaped.
fn write_manifest_notices(out: &mut dyn Write, coverage: &types::CoverageReport) {
    for notice in &coverage.manifest_notices {
        let _ = writeln!(out, "{} {notice}", "⚠".yellow());
    }
}

pub fn print_coverage_report(coverage: &types::CoverageReport) {
    println!(
        "\n--- {} ------------------------------------------------",
        "Coverage Report".bold()
    );

    if coverage.total_source_files == 0 {
        // Vacuous for the same reason: no modules were found to have or lack
        // spec directories.
    } else if coverage.unspecced_modules.is_empty() {
        println!(
            "\n  {} All source modules have spec directories",
            "✓".green()
        );
    } else {
        println!(
            "\n  Modules without specs ({}):",
            coverage.unspecced_modules.len()
        );
        for module in &coverage.unspecced_modules {
            println!("    {} {module}/", "⚠".yellow());
        }
    }

    // Files a spec references but that do not exist on disk must never sit
    // under a green "all referenced" line — name them as failures instead.
    if !coverage.missing_files.is_empty() {
        println!(
            "\n  Referenced by specs but missing on disk ({}):",
            coverage.missing_files.len()
        );
        for file in &coverage.missing_files {
            println!("    {} {file}", "✗".red());
        }
    }

    if coverage.total_source_files == 0 {
        // Both lines below are true of an empty set and read as measurements.
        println!(
            "  {} No source files were found to measure — check `source_dirs` and `exclude_patterns`",
            "⊘".yellow()
        );
    } else if coverage.unspecced_files.is_empty() && coverage.missing_files.is_empty() {
        println!("  {} All source files referenced by specs", "✓".green());
    } else if !coverage.unspecced_files.is_empty() {
        let uncovered_loc: usize = coverage.unspecced_file_loc.iter().map(|(_, l)| l).sum();
        println!(
            "\n  Files not in any spec ({}, {} LOC uncovered):",
            coverage.unspecced_files.len(),
            uncovered_loc
        );
        for (file, loc) in &coverage.unspecced_file_loc {
            println!("    {} {file} ({loc} LOC)", "⚠".yellow());
        }
    }
}

#[allow(clippy::too_many_arguments)]
pub fn print_check_markdown(
    total: usize,
    passed: usize,
    warnings: usize,
    errors: usize,
    all_errors: &[String],
    all_warnings: &[String],
    all_notices: &[String],
    coverage: &types::CoverageReport,
    overall_passed: bool,
) {
    let status = if overall_passed { "Passed" } else { "Failed" };
    let icon = if overall_passed { "✅" } else { "❌" };

    println!("## SpecSync Check Results\n");
    println!(
        "**{icon} {status}** — {total} specs checked, {passed} passed, {warnings} warning(s), {errors} error(s)\n"
    );

    if !all_errors.is_empty() {
        println!("### Errors\n");
        for e in all_errors {
            println!("- {e}");
        }
        println!();
    }

    if !all_warnings.is_empty() {
        println!("### Warnings\n");
        for w in all_warnings {
            println!("- {w}");
        }
        println!();
    }

    if !all_notices.is_empty() {
        println!("### Planned Mappings\n");
        for notice in all_notices {
            println!("- {notice}");
        }
        println!();
    }

    println!("### Coverage\n");
    // Same rule as the text renderer, and now the same source: a zero
    // denominator has no percentage to print. This renderer used to print
    // `0/0 (100%)` while the text one printed "no source files to measure"
    // for the very same report (#582).
    match coverage.file_coverage_percent() {
        Some(pct) => println!(
            "- **Files:** {}/{} ({pct}%)",
            coverage.specced_file_count,
            coverage.measured_file_total()
        ),
        None => println!("- **Files:** 0/0 ({NO_FILES_MEASURED})"),
    }
    match coverage.loc_coverage_percent() {
        Some(pct) => println!(
            "- **LOC:** {}/{} ({pct}%)",
            coverage.specced_loc, coverage.total_loc
        ),
        None => println!("- **LOC:** 0/0 ({NO_LINES_MEASURED})"),
    }
    // Absent files a spec claims are part of the file total (they can never be
    // covered, so a gate must not ignore them), which makes the total exceed
    // what is on disk. Named here so the total is never read without them.
    if !coverage.missing_files.is_empty() {
        println!(
            "- **Referenced but missing on disk (counted in the total):** {}",
            coverage
                .missing_files
                .iter()
                .map(|file| format!("`{file}`"))
                .collect::<Vec<_>>()
                .join(", ")
        );
    }
    // Same reason as the text renderer: these percentages are measured over
    // whatever was left after skipping links (#546), so the exclusion belongs
    // with the numbers rather than in a separate section a reader may not reach.
    if !coverage.skipped_links.is_empty() {
        println!(
            "- **Excluded (symlinks, not traversed):** {}",
            coverage
                .skipped_links
                .iter()
                .map(|link| format!("`{link}`"))
                .collect::<Vec<_>>()
                .join(", ")
        );
    }
    // Same placement, same reason (#723): a manifest that could not be parsed
    // still declared modules, so the module figures above were measured over
    // less than the tree holds.
    for notice in &coverage.manifest_notices {
        println!("- **Manifest discovery degraded:** {notice}");
    }
}

/// Print diff results as markdown. Each entry is (spec, changed_files, new_exports, removed_exports).
#[allow(clippy::type_complexity)]
pub fn print_diff_markdown(
    entries: &[(String, Vec<String>, Vec<String>, Vec<String>, bool)],
    changed_files: &std::collections::HashSet<String>,
    spec_files: &[std::path::PathBuf],
    _root: &std::path::Path,
    config: &types::SpecSyncConfig,
    base: &str,
) {
    println!("## SpecSync Drift Report\n");

    if entries.is_empty() {
        // Check for untracked files
        let specced_files: std::collections::HashSet<String> = spec_files
            .iter()
            .filter_map(|f| std::fs::read_to_string(f).ok())
            .filter_map(|c| crate::parser::parse_frontmatter(&c.replace("\r\n", "\n")))
            .flat_map(|p| p.frontmatter.files)
            .collect();

        let untracked: Vec<&String> = changed_files
            .iter()
            .filter(|f| {
                let path = std::path::Path::new(f.as_str());
                crate::exports::has_configured_extension(
                    path,
                    &config.source_extensions,
                    config.include_extensionless,
                ) && !specced_files.contains(*f)
            })
            .collect();

        if untracked.is_empty() {
            println!("No spec-tracked source files changed since `{base}`.");
        } else {
            println!("**Changed files not covered by any spec:**\n");
            for f in &untracked {
                println!("- `{f}`");
            }
        }
        return;
    }

    let has_drift = entries
        .iter()
        .any(|(_, _, new, removed, _)| !new.is_empty() || !removed.is_empty());

    if has_drift {
        println!(
            "Spec drift detected in {} module(s) since `{base}`.\n",
            entries.len()
        );
    } else {
        println!("All specs are up to date with source code.\n");
    }

    for (spec, files, new_exports, removed_exports, spec_modified) in entries {
        println!("### `{spec}`\n");
        if *spec_modified {
            println!("**Spec file modified in this PR.**\n");
        }
        if !files.is_empty() {
            println!(
                "**Changed source files:** {}\n",
                files
                    .iter()
                    .map(|f| format!("`{f}`"))
                    .collect::<Vec<_>>()
                    .join(", ")
            );
        }

        if !new_exports.is_empty() || !removed_exports.is_empty() {
            println!("| Change | Export |");
            println!("|--------|--------|");
            for e in new_exports {
                println!("| Added | `{e}` |");
            }
            for e in removed_exports {
                println!("| Removed | `{e}` |");
            }
            println!();
        } else {
            println!("No drift — spec is up to date.\n");
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A report whose percentages work out to `file_pct` / `loc_pct` over a
    /// hundred-file, hundred-line tree.
    fn coverage(file_pct: usize, loc_pct: usize) -> types::CoverageReport {
        types::CoverageReport {
            total_source_files: 100,
            specced_file_count: file_pct,
            unspecced_files: Vec::new(),
            unspecced_modules: Vec::new(),
            total_loc: 100,
            specced_loc: loc_pct,
            unspecced_file_loc: Vec::new(),
            missing_files: Vec::new(),
            skipped_links: Vec::new(),
            manifest_notices: Vec::new(),
        }
    }

    #[test]
    fn print_summary_does_not_underflow_when_passed_exceeds_total() {
        // Regression: `total - passed` used to panic on underflow in debug.
        print_summary(2, 5, 0, 0);
    }

    #[test]
    fn print_summary_handles_zero_and_all_passed() {
        print_summary(0, 0, 0, 0);
        print_summary(3, 3, 0, 0);
    }

    #[test]
    fn print_coverage_line_handles_color_threshold_boundaries() {
        // Exercises each color branch: <80 (red), ==80 (yellow), 100 (green).
        for pct in [0usize, 79, 80, 99, 100] {
            print_coverage_line(&coverage(pct, pct));
        }
    }

    #[test]
    fn percent_json_reports_absence_as_null_and_a_measurement_as_a_number() {
        // Machine consumers get `null` for "not measured" — the same shape the
        // inconclusive-discovery payloads already use — and never a fabricated
        // 100.0 (#582).
        assert!(percent_json(None).is_null());
        assert_eq!(percent_json(Some(0.0)), serde_json::json!(0.0));
        assert_eq!(percent_json(Some(100.0)), serde_json::json!(100.0));
        assert_eq!(
            percent_json(Some(66.666_666)),
            serde_json::json!(66.67),
            "measured values keep their hundredths rounding"
        );
    }
}
