//! `specsync doctor` (REQ-cmd-change-035): print what the scan found, what `--fix` did and what
//! `--dry-run` would do. A submodule of the change commands; the scan and the fixes live in
//! `crate::change`.

use colored::Colorize;
use std::path::Path;
use std::process;

use crate::change::{self, DoctorFixClass, DoctorReport, DoctorRequest};
use crate::types::OutputFormat;

/// Run doctor for `root` and print its report in `format`. Exits 1 when a late close is refused or
/// a fix it attempted failed; a scan that finds problems still exits 0.
pub fn cmd_doctor(root: &Path, request: DoctorRequest, format: OutputFormat) {
    // Repository facts git answers during this command are asked once; git writes clear them.
    let _facts = crate::git_utils::begin_repository_fact_scope(root);
    match change::run_doctor(root, &request) {
        Ok(report) => {
            match format {
                OutputFormat::Json => match serde_json::to_string_pretty(&report) {
                    Ok(json) => println!("{json}"),
                    Err(error) => {
                        eprintln!("{} {error}", "error:".red().bold());
                        process::exit(1);
                    }
                },
                _ => print_text(&report),
            }
            if !report.is_clean() {
                process::exit(1);
            }
        }
        Err(error) => {
            match format {
                OutputFormat::Json => println!("{}", serde_json::json!({ "error": error })),
                _ => eprintln!("{} {error}", "error:".red().bold()),
            }
            process::exit(1);
        }
    }
}

fn class_label(class: DoctorFixClass) -> &'static str {
    match class {
        DoctorFixClass::Safe => "safe: `doctor --fix` applies it",
        DoctorFixClass::Decision => "decision: needs --actor and --reason",
        DoctorFixClass::ReportOnly => "report only",
    }
}

fn print_text(report: &DoctorReport) {
    if report.findings.is_empty() {
        println!("specsync doctor: no problems found in the change records.");
    } else {
        println!(
            "specsync doctor: {} finding(s) in the change records",
            report.findings.len()
        );
        for finding in &report.findings {
            println!(
                "\n  {} {} ({})",
                finding.kind.bold(),
                finding.change.as_deref().unwrap_or("project"),
                finding.location
            );
            println!("    {}", finding.reason);
            println!(
                "    fix ({}): {}",
                class_label(finding.fix.class),
                finding.fix.command
            );
            if let Some(alternative) = &finding.fix.alternative {
                println!("    or: {alternative}");
            }
        }
    }
    if !report.actions.is_empty() {
        println!();
        if report.dry_run {
            println!("{}", "Dry run: nothing was written.".bold());
        }
        for action in &report.actions {
            println!(
                "  {} {} {}: {}",
                action.outcome,
                action.kind,
                action.change.as_deref().unwrap_or("project"),
                action.detail
            );
        }
    }
}
