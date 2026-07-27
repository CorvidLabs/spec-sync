use colored::Colorize;
use std::fmt::Write as _;
use std::io::{self, Write as _};
use std::path::Path;
use std::process;

use crate::compact;
use crate::config::load_config;
use crate::types;

pub fn cmd_compact(root: &Path, keep: usize, dry_run: bool, format: types::OutputFormat) {
    let config = load_config(root);
    let specs_dir = root.join(&config.specs_dir);
    let report = compact::compact_changelogs(root, &specs_dir, keep, dry_run);

    match format {
        types::OutputFormat::Json => print_json(&report, keep, dry_run),
        types::OutputFormat::Markdown | types::OutputFormat::Github => {
            print_markdown(&report, dry_run)
        }
        types::OutputFormat::Text | types::OutputFormat::Table | types::OutputFormat::Csv => {
            print_text(&report, dry_run)
        }
    }

    if !report.complete() {
        let _ = io::stdout().flush();
        process::exit(1);
    }
}

fn print_text(report: &compact::CompactReport, dry_run: bool) {
    if dry_run {
        println!("{} Dry run — no files will be modified\n", "ℹ".cyan());
    }

    if report.results.is_empty() && report.failures.is_empty() {
        println!(
            "{}",
            "No changelogs need compaction (all within limit).".green()
        );
        return;
    }

    for result in &report.results {
        let verb = if dry_run {
            "would compact"
        } else if result.applied {
            "compacted"
        } else {
            "not applied"
        };
        println!(
            "  {} {} — {verb} {} {} (kept {})",
            if dry_run || result.applied {
                "✓".green()
            } else {
                "✗".red()
            },
            safe_diagnostic(&result.spec_path),
            result.removed,
            if result.removed == 1 {
                "entry"
            } else {
                "entries"
            },
            result.compacted_entries,
        );
    }

    for failure in &report.failures {
        eprintln!(
            "{} {} ({}): {}",
            "error:".red().bold(),
            safe_diagnostic(&failure.spec_path),
            failure.operation,
            safe_diagnostic(&failure.message)
        );
    }

    let total_removed: usize = report.results.iter().map(|result| result.removed).sum();
    println!(
        "\n{} {} {} across {} {}",
        if dry_run {
            "Would compact".to_string()
        } else if report.complete() {
            "Compacted".to_string()
        } else {
            "Planned".to_string()
        },
        total_removed,
        if total_removed == 1 {
            "entry"
        } else {
            "entries"
        },
        report.results.len(),
        if report.results.len() == 1 {
            "spec"
        } else {
            "specs"
        }
    );
}

fn print_json(report: &compact::CompactReport, keep: usize, dry_run: bool) {
    let entries_affected: usize = report.results.iter().map(|result| result.removed).sum();
    let entries_applied: usize = report
        .results
        .iter()
        .filter(|result| result.applied)
        .map(|result| result.removed)
        .sum();
    let rendered_results: Vec<serde_json::Value> = report
        .results
        .iter()
        .map(|result| {
            serde_json::json!({
                "spec_path": portable_output_path(&result.spec_path),
                "action": if dry_run {
                    "would_compact"
                } else if result.applied {
                    "compacted"
                } else {
                    "not_applied"
                },
                "applied": result.applied,
                "original_entries": result.original_entries,
                "kept_entries": result.compacted_entries,
                "entries_affected": result.removed,
            })
        })
        .collect();
    let failures: Vec<serde_json::Value> = report
        .failures
        .iter()
        .map(|failure| {
            serde_json::json!({
                "spec_path": portable_output_path(&failure.spec_path),
                "operation": failure.operation,
                "error": failure.message,
            })
        })
        .collect();
    let would_change = report.planned > 0;
    let output = serde_json::json!({
        "command": "compact",
        "dry_run": dry_run,
        "keep": keep,
        "would_change": would_change,
        "applied": would_change && !dry_run && report.complete() && report.succeeded == report.planned,
        "complete": report.complete(),
        "partial": report.partial(),
        "operations": {
            "planned": report.planned,
            "succeeded": report.succeeded,
            "failed": report.failures.len(),
        },
        "entries_affected": entries_affected,
        "entries_applied": entries_applied,
        "specs_affected": report.results.len(),
        "results": rendered_results,
        "errors": failures,
    });
    println!("{}", serde_json::to_string_pretty(&output).unwrap());
}

fn print_markdown(report: &compact::CompactReport, dry_run: bool) {
    println!("## SpecSync Compact Results\n");
    if dry_run {
        println!("> Dry run — no files will be modified.\n");
    }

    if report.results.is_empty() && report.failures.is_empty() {
        println!("No changelogs need compaction (all within limit).");
        return;
    }

    if !report.results.is_empty() {
        println!("| Spec | Action | Entries affected | Kept |");
        println!("|------|--------|-----------------:|-----:|");
        for result in &report.results {
            let action = if dry_run {
                "Would compact"
            } else if result.applied {
                "Compacted"
            } else {
                "Not applied"
            };
            println!(
                "| {} | {action} | {} | {} |",
                markdown_code_span(&portable_output_path(&result.spec_path)),
                result.removed,
                result.compacted_entries,
            );
        }
    }

    if !report.failures.is_empty() {
        println!("\n| Failed path | Operation | Error |");
        println!("|-------------|-----------|-------|");
        for failure in &report.failures {
            println!(
                "| {} | {} | {} |",
                markdown_code_span(&portable_output_path(&failure.spec_path)),
                markdown_cell(failure.operation),
                markdown_cell(&failure.message),
            );
        }
    }

    let entries_affected: usize = report.results.iter().map(|result| result.removed).sum();
    let action = if dry_run {
        "Would compact"
    } else if report.complete() {
        "Compacted"
    } else {
        "Planned"
    };
    println!(
        "\n**Summary:** {action} {entries_affected} {} across {} {}.",
        if entries_affected == 1 {
            "entry"
        } else {
            "entries"
        },
        report.results.len(),
        if report.results.len() == 1 {
            "spec"
        } else {
            "specs"
        },
    );
}

fn is_unsafe_diagnostic_character(character: char) -> bool {
    character.is_control()
        || matches!(
            character,
            '\u{061c}'
                | '\u{200e}'
                | '\u{200f}'
                | '\u{2028}'
                | '\u{2029}'
                | '\u{202a}'..='\u{202e}'
                | '\u{2066}'..='\u{2069}'
        )
}

fn safe_diagnostic(value: &str) -> String {
    let mut safe = String::with_capacity(value.len());
    for character in value.chars() {
        if is_unsafe_diagnostic_character(character) {
            write!(&mut safe, "\\u{{{:04X}}}", character as u32)
                .expect("writing to a String cannot fail");
        } else {
            safe.push(character);
        }
    }
    safe
}

fn markdown_cell(value: &str) -> String {
    safe_diagnostic(value)
        .replace('\\', "\\\\")
        .replace('|', "\\|")
}

fn markdown_code_span(value: &str) -> String {
    let value = markdown_cell(value);
    let longest_backtick_run = value
        .split(|character| character != '`')
        .map(str::len)
        .max()
        .unwrap_or(0);
    let delimiter = "`".repeat(longest_backtick_run + 1);
    if value.starts_with('`') || value.ends_with('`') {
        format!("{delimiter} {value} {delimiter}")
    } else {
        format!("{delimiter}{value}{delimiter}")
    }
}

fn portable_output_path(value: &str) -> String {
    #[cfg(windows)]
    {
        value.replace('\\', "/")
    }
    #[cfg(not(windows))]
    {
        value.to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::{markdown_code_span, portable_output_path};

    #[cfg(windows)]
    #[test]
    fn structured_output_paths_use_portable_separators() {
        assert_eq!(
            portable_output_path(r"specs\history\history.spec.md"),
            "specs/history/history.spec.md"
        );
    }

    #[cfg(not(windows))]
    #[test]
    fn structured_output_paths_preserve_literal_backslashes_on_unix() {
        assert_eq!(
            portable_output_path(r"specs/history\literal/history.spec.md"),
            r"specs/history\literal/history.spec.md"
        );
    }

    #[test]
    fn markdown_code_span_sanitizes_paths_and_uses_safe_delimiters() {
        assert_eq!(
            markdown_code_span("bad`name|row\n.spec.md"),
            "``bad`name\\|row\\\\u{000A}.spec.md``"
        );
        assert_eq!(markdown_code_span("``edge"), "``` ``edge ```");
    }
}
