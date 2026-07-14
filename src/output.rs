use colored::Colorize;

use crate::types;

pub fn print_summary(total: usize, passed: usize, warnings: usize, _errors: usize) {
    // saturating_sub guards against an underflow panic if `passed` is ever
    // reported higher than `total`.
    let failed = total.saturating_sub(passed);
    println!(
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

pub fn print_coverage_line(coverage: &types::CoverageReport) {
    let pct = coverage.coverage_percent;
    let pct_str = format!("{pct}%");
    let colored_pct = if pct == 100 {
        pct_str.green().to_string()
    } else if pct >= 80 {
        pct_str.yellow().to_string()
    } else {
        pct_str.red().to_string()
    };

    let loc_pct = coverage.loc_coverage_percent;
    let loc_pct_str = format!("{loc_pct}%");
    let colored_loc_pct = if loc_pct == 100 {
        loc_pct_str.green().to_string()
    } else if loc_pct >= 80 {
        loc_pct_str.yellow().to_string()
    } else {
        loc_pct_str.red().to_string()
    };

    println!(
        "File coverage: {}/{} ({colored_pct})",
        coverage.specced_file_count, coverage.total_source_files
    );
    println!(
        "LOC coverage:  {}/{} ({colored_loc_pct})",
        coverage.specced_loc, coverage.total_loc
    );
}

pub fn print_coverage_report(coverage: &types::CoverageReport) {
    println!(
        "\n--- {} ------------------------------------------------",
        "Coverage Report".bold()
    );

    if coverage.unspecced_modules.is_empty() {
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

    if coverage.unspecced_files.is_empty() {
        println!("  {} All source files referenced by specs", "✓".green());
    } else {
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
    println!(
        "- **Files:** {}/{} ({}%)",
        coverage.specced_file_count, coverage.total_source_files, coverage.coverage_percent
    );
    println!(
        "- **LOC:** {}/{} ({}%)",
        coverage.specced_loc, coverage.total_loc, coverage.loc_coverage_percent
    );
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

    fn coverage(file_pct: usize, loc_pct: usize) -> types::CoverageReport {
        types::CoverageReport {
            total_source_files: 1,
            specced_file_count: 1,
            unspecced_files: Vec::new(),
            unspecced_modules: Vec::new(),
            coverage_percent: file_pct,
            total_loc: 1,
            specced_loc: 1,
            loc_coverage_percent: loc_pct,
            unspecced_file_loc: Vec::new(),
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
}
