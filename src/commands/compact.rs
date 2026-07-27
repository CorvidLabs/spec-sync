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
    println!("{}", render_json(report, keep, dry_run));
}

fn render_json(report: &compact::CompactReport, keep: usize, dry_run: bool) -> String {
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
    serde_json::to_string_pretty(&output).unwrap()
}

fn print_markdown(report: &compact::CompactReport, dry_run: bool) {
    print!("{}", render_markdown(report, dry_run));
}

fn render_markdown(report: &compact::CompactReport, dry_run: bool) -> String {
    let mut output = String::new();
    writeln!(&mut output, "## SpecSync Compact Results\n")
        .expect("writing to a String cannot fail");
    if dry_run {
        writeln!(&mut output, "> Dry run — no files will be modified.\n")
            .expect("writing to a String cannot fail");
    }

    if report.results.is_empty() && report.failures.is_empty() {
        writeln!(
            &mut output,
            "No changelogs need compaction (all within limit)."
        )
        .expect("writing to a String cannot fail");
        return output;
    }

    if !report.results.is_empty() {
        writeln!(&mut output, "| Spec | Action | Entries affected | Kept |")
            .expect("writing to a String cannot fail");
        writeln!(&mut output, "|------|--------|-----------------:|-----:|")
            .expect("writing to a String cannot fail");
        for result in &report.results {
            let action = if dry_run {
                "Would compact"
            } else if result.applied {
                "Compacted"
            } else {
                "Not applied"
            };
            writeln!(
                &mut output,
                "| {} | {action} | {} | {} |",
                markdown_code_span(&portable_output_path(&result.spec_path)),
                result.removed,
                result.compacted_entries,
            )
            .expect("writing to a String cannot fail");
        }
    }

    if !report.failures.is_empty() {
        writeln!(&mut output, "\n| Failed path | Operation | Error |")
            .expect("writing to a String cannot fail");
        writeln!(&mut output, "|-------------|-----------|-------|")
            .expect("writing to a String cannot fail");
        for failure in &report.failures {
            writeln!(
                &mut output,
                "| {} | {} | {} |",
                markdown_code_span(&portable_output_path(&failure.spec_path)),
                markdown_cell(failure.operation),
                markdown_cell(&failure.message),
            )
            .expect("writing to a String cannot fail");
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
    writeln!(
        &mut output,
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
    )
    .expect("writing to a String cannot fail");
    output
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
    markdown_html_text(&safe_diagnostic(value))
}

fn markdown_code_span(value: &str) -> String {
    let value = safe_diagnostic(value);
    if value.contains('|') {
        return format!("<code>{}</code>", markdown_html_text(&value));
    }

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

fn markdown_html_text(value: &str) -> String {
    value
        .replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('\\', "&#92;")
        .replace('|', "&#124;")
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
    use super::{markdown_code_span, portable_output_path, render_json, render_markdown};
    use crate::compact::{CompactFailure, CompactReport, CompactResult};

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
            "<code>bad`name&#124;row&#92;u{000A}.spec.md</code>"
        );
        assert_eq!(
            markdown_code_span("bad</code>&|row.spec.md"),
            "<code>bad&lt;/code&gt;&amp;&#124;row.spec.md</code>"
        );
        assert_eq!(markdown_code_span("``edge"), "``` ``edge ```");
    }

    #[cfg(unix)]
    #[test]
    fn markdown_code_span_preserves_literal_unix_backslashes() {
        assert_eq!(
            markdown_code_span(r"specs/\\server\share/history.spec.md"),
            r"`specs/\\server\share/history.spec.md`"
        );
    }

    #[cfg(unix)]
    #[test]
    fn markdown_code_span_preserves_backslash_before_pipe_in_one_table_cell() {
        assert_eq!(
            markdown_code_span(r"specs/a\|b/history.spec.md"),
            r"<code>specs/a&#92;&#124;b/history.spec.md</code>"
        );
    }

    #[test]
    fn partial_publish_reports_are_truthful_in_json_and_markdown() {
        let report = CompactReport {
            results: vec![
                CompactResult {
                    spec_path: "specs/a/a.spec.md".to_string(),
                    original_entries: 4,
                    compacted_entries: 2,
                    removed: 2,
                    applied: true,
                },
                CompactResult {
                    spec_path: "specs/b/b.spec.md".to_string(),
                    original_entries: 4,
                    compacted_entries: 2,
                    removed: 2,
                    applied: false,
                },
            ],
            failures: vec![CompactFailure {
                spec_path: "specs/b/b.spec.md".to_string(),
                operation: "publish",
                message: "publication failed".to_string(),
            }],
            planned: 2,
            succeeded: 1,
        };

        let json: serde_json::Value = serde_json::from_str(&render_json(&report, 2, false))
            .expect("partial JSON must be parseable");
        assert_eq!(json["complete"], false);
        assert_eq!(json["partial"], true);
        assert_eq!(json["applied"], false);
        assert_eq!(json["entries_affected"], 4);
        assert_eq!(json["entries_applied"], 2);
        assert_eq!(json["specs_affected"], 2);
        assert_eq!(json["operations"]["planned"], 2);
        assert_eq!(json["operations"]["succeeded"], 1);
        assert_eq!(json["errors"][0]["operation"], "publish");

        let markdown = render_markdown(&report, false);
        assert!(markdown.contains("| `specs/a/a.spec.md` | Compacted | 2 | 2 |"));
        assert!(markdown.contains("| `specs/b/b.spec.md` | Not applied | 2 | 2 |"));
        assert!(markdown.contains("| `specs/b/b.spec.md` | publish | publication failed |"));
        assert!(markdown.contains("**Summary:** Planned 4 entries across 2 specs."));
    }
}
