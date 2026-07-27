use colored::Colorize;
use std::fmt::Write as _;
use std::io::{self, Write as _};
use std::path::Path;
use std::process;

use crate::archive;
use crate::config::load_config;
use crate::types;

pub fn cmd_archive_tasks(root: &Path, dry_run: bool, format: types::OutputFormat) {
    let config = load_config(root);
    let specs_dir = root.join(&config.specs_dir);
    let report = archive::archive_tasks(root, &specs_dir, dry_run);

    let output = match format {
        types::OutputFormat::Json => render_json(&report),
        types::OutputFormat::Markdown | types::OutputFormat::Github => render_markdown(&report),
        types::OutputFormat::Text | types::OutputFormat::Table | types::OutputFormat::Csv => {
            render_text(&report)
        }
    };
    print!("{output}");

    if !report.is_complete() {
        // `process::exit` does not run normal destructors, so flush the already-rendered report
        // before returning the documented findings/inconclusive exit code.
        let _ = io::stdout().flush();
        process::exit(1);
    }
}

fn render_text(report: &archive::ArchiveReport) -> String {
    let mut output = String::new();
    if report.dry_run {
        writeln!(
            output,
            "{} Dry run — no files will be modified\n",
            "ℹ".cyan()
        )
        .expect("writing to a String cannot fail");
    }

    if report.planned.is_empty() && report.failed.is_empty() {
        writeln!(output, "{}", "No completed tasks to archive.".green())
            .expect("writing to a String cannot fail");
        return output;
    }

    let visible_results = if report.dry_run {
        &report.planned
    } else {
        &report.succeeded
    };
    for result in visible_results {
        let verb = if report.dry_run {
            "would archive"
        } else {
            "archived"
        };
        writeln!(
            output,
            "  {} {} — {verb} {} {}",
            "✓".green(),
            safe_diagnostic(&result.tasks_path.to_string_lossy()),
            result.archived_count,
            plural(result.archived_count, "task", "tasks"),
        )
        .expect("writing to a String cannot fail");
    }

    if !report.dry_run && !report.is_complete() {
        for planned in report.planned.iter().filter(|planned| {
            !report
                .succeeded
                .iter()
                .any(|succeeded| succeeded.tasks_path == planned.tasks_path)
        }) {
            writeln!(
                output,
                "  {} {} — planned {} {}, not applied",
                "○".yellow(),
                safe_diagnostic(&planned.tasks_path.to_string_lossy()),
                planned.archived_count,
                plural(planned.archived_count, "task", "tasks"),
            )
            .expect("writing to a String cannot fail");
        }
    }

    for failure in &report.failed {
        writeln!(
            output,
            "  {} {} — {} failed: {}",
            "✗".red(),
            safe_diagnostic(&failure.tasks_path.to_string_lossy()),
            failure.operation.as_str(),
            safe_diagnostic(&failure.error),
        )
        .expect("writing to a String cannot fail");
    }

    if !report.rolled_back.is_empty() {
        writeln!(
            output,
            "\nRolled back {} {} after a publication failure.",
            report.rolled_back.len(),
            plural(report.rolled_back.len(), "file", "files"),
        )
        .expect("writing to a String cannot fail");
    }

    if report.is_complete() {
        let tasks = report.planned_tasks();
        writeln!(
            output,
            "\n{} {} {} across {} {}",
            if report.dry_run {
                "Would archive"
            } else {
                "Archived"
            },
            tasks,
            plural(tasks, "task", "tasks"),
            report.planned.len(),
            plural(report.planned.len(), "file", "files"),
        )
        .expect("writing to a String cannot fail");
    } else {
        writeln!(
            output,
            "\nArchive incomplete: {} {} planned, {} {} archived; {} {} failed.",
            report.planned_tasks(),
            plural(report.planned_tasks(), "task", "tasks"),
            report.succeeded_tasks(),
            plural(report.succeeded_tasks(), "task", "tasks"),
            report.failed.len(),
            plural(report.failed.len(), "operation", "operations"),
        )
        .expect("writing to a String cannot fail");
    }

    output
}

fn render_json(report: &archive::ArchiveReport) -> String {
    let render_result = |result: &archive::ArchiveResult, action: &str| {
        serde_json::json!({
            "tasks_path": portable_output_path(&result.tasks_path),
            "operation": "archive",
            "action": action,
            "tasks_affected": result.archived_count,
        })
    };
    let planned_action = if report.dry_run {
        "would_archive"
    } else {
        "planned"
    };
    let planned: Vec<_> = report
        .planned
        .iter()
        .map(|result| render_result(result, planned_action))
        .collect();
    let succeeded: Vec<_> = report
        .succeeded
        .iter()
        .map(|result| render_result(result, "archived"))
        .collect();
    let rolled_back: Vec<_> = report
        .rolled_back
        .iter()
        .map(|result| render_result(result, "rolled_back"))
        .collect();
    let failed: Vec<_> = report
        .failed
        .iter()
        .map(|failure| {
            serde_json::json!({
                "tasks_path": portable_output_path(&failure.tasks_path),
                "operation": failure.operation.as_str(),
                "error": failure.error,
            })
        })
        .collect();
    let results = if report.dry_run {
        planned.clone()
    } else {
        succeeded.clone()
    };

    let output = serde_json::json!({
        "command": "archive-tasks",
        "dry_run": report.dry_run,
        "would_change": !report.planned.is_empty(),
        "applied": report.applied(),
        "complete": report.is_complete(),
        "partial": report.is_partial(),
        "tasks_affected": if report.dry_run {
            report.planned_tasks()
        } else {
            report.succeeded_tasks()
        },
        "files_affected": results.len(),
        "tasks_planned": report.planned_tasks(),
        "tasks_succeeded": report.succeeded_tasks(),
        "files_planned": report.planned.len(),
        "files_succeeded": report.succeeded.len(),
        "planned": planned,
        "succeeded": succeeded,
        "rolled_back": rolled_back,
        "failed": failed,
        "results": results,
    });
    format!(
        "{}\n",
        serde_json::to_string_pretty(&output).expect("archive report is JSON serializable")
    )
}

fn render_markdown(report: &archive::ArchiveReport) -> String {
    let mut output = String::from("## SpecSync Archive Tasks Results\n\n");
    if report.dry_run {
        output.push_str("> Dry run — no files will be modified.\n\n");
    }

    if report.planned.is_empty() && report.failed.is_empty() {
        output.push_str("No completed tasks to archive.\n");
        return output;
    }

    let visible_results = if report.dry_run {
        &report.planned
    } else {
        &report.succeeded
    };
    if !visible_results.is_empty() {
        output.push_str("| Tasks file | Action | Tasks affected |\n");
        output.push_str("|------------|--------|---------------:|\n");
        let action = if report.dry_run {
            "Would archive"
        } else {
            "Archived"
        };
        for result in visible_results {
            writeln!(
                output,
                "| {} | {action} | {} |",
                markdown_code_span(&portable_output_path(&result.tasks_path)),
                result.archived_count,
            )
            .expect("writing to a String cannot fail");
        }
        output.push('\n');
    }

    if !report.failed.is_empty() {
        output.push_str("| Tasks file | Failed operation | Error |\n");
        output.push_str("|------------|------------------|-------|\n");
        for failure in &report.failed {
            writeln!(
                output,
                "| {} | {} | {} |",
                markdown_code_span(&portable_output_path(&failure.tasks_path)),
                failure.operation.as_str(),
                markdown_table_cell(&failure.error),
            )
            .expect("writing to a String cannot fail");
        }
        output.push('\n');
    }

    if !report.rolled_back.is_empty() {
        writeln!(
            output,
            "> Rolled back {} {} after a publication failure.\n",
            report.rolled_back.len(),
            plural(report.rolled_back.len(), "file", "files"),
        )
        .expect("writing to a String cannot fail");
    }

    if report.is_complete() {
        let tasks = report.planned_tasks();
        writeln!(
            output,
            "**Summary:** {} {tasks} {} across {} {}.",
            if report.dry_run {
                "Would archive"
            } else {
                "Archived"
            },
            plural(tasks, "task", "tasks"),
            report.planned.len(),
            plural(report.planned.len(), "file", "files"),
        )
        .expect("writing to a String cannot fail");
    } else {
        writeln!(
            output,
            "**Summary:** Archive incomplete — {} {} planned, {} {} archived; {} {} failed.",
            report.planned_tasks(),
            plural(report.planned_tasks(), "task", "tasks"),
            report.succeeded_tasks(),
            plural(report.succeeded_tasks(), "task", "tasks"),
            report.failed.len(),
            plural(report.failed.len(), "operation", "operations"),
        )
        .expect("writing to a String cannot fail");
    }

    output
}

fn plural<'a>(count: usize, singular: &'a str, plural: &'a str) -> &'a str {
    if count == 1 { singular } else { plural }
}

fn portable_output_path(value: &Path) -> String {
    let rendered = value.to_string_lossy().into_owned();
    #[cfg(windows)]
    {
        rendered.replace('\\', "/")
    }
    #[cfg(not(windows))]
    {
        rendered
    }
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

fn markdown_table_cell(value: &str) -> String {
    safe_diagnostic(value)
        .replace('\\', "\\\\")
        .replace('|', "\\|")
}

fn markdown_code_span(value: &str) -> String {
    let value = markdown_table_cell(value);
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

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    fn report_for(path: PathBuf, count: usize, dry_run: bool) -> archive::ArchiveReport {
        let result = archive::ArchiveResult {
            tasks_path: path,
            archived_count: count,
        };
        archive::ArchiveReport {
            dry_run,
            planned: vec![result.clone()],
            succeeded: if dry_run { Vec::new() } else { vec![result] },
            rolled_back: Vec::new(),
            failed: Vec::new(),
        }
    }

    #[test]
    fn text_uses_truthful_singular_and_plural_labels() {
        let singular = render_text(&report_for(PathBuf::from("specs/one/tasks.md"), 1, true));
        assert!(singular.contains("would archive 1 task"));
        assert!(singular.contains("Would archive 1 task across 1 file"));
        assert!(!singular.contains("task(s)"));
        assert!(!singular.contains("file(s)"));

        let plural = render_text(&report_for(PathBuf::from("specs/many/tasks.md"), 2, false));
        assert!(plural.contains("archived 2 tasks"));
        assert!(plural.contains("Archived 2 tasks across 1 file"));
    }

    #[cfg(unix)]
    #[test]
    fn structured_output_preserves_literal_unix_backslashes() {
        let path = Path::new(r"specs/literal\name/tasks.md");
        assert_eq!(portable_output_path(path), r"specs/literal\name/tasks.md");
    }

    #[cfg(windows)]
    #[test]
    fn structured_output_normalizes_windows_separators() {
        let path = Path::new(r"specs\work\tasks.md");
        assert_eq!(portable_output_path(path), "specs/work/tasks.md");
    }

    #[test]
    fn markdown_paths_use_safe_dynamic_code_spans() {
        let path = PathBuf::from("specs/pipe|ticks``\n\u{0007}\u{202E}/tasks.md");
        let markdown = render_markdown(&report_for(path, 1, true));

        assert!(markdown.contains("\\|"));
        assert!(markdown.contains("\\u{000A}"));
        assert!(markdown.contains("\\u{0007}"));
        assert!(markdown.contains("\\u{202E}"));
        assert!(markdown.contains("```"));
        assert_eq!(
            markdown
                .lines()
                .filter(|line| line.contains("Would archive") && line.starts_with('|'))
                .count(),
            1,
            "adversarial path injected a Markdown row"
        );
    }

    #[test]
    fn json_safely_escapes_control_characters_in_paths_and_errors() {
        let result = archive::ArchiveResult {
            tasks_path: PathBuf::from("specs/new\nline/tasks.md"),
            archived_count: 1,
        };
        let report = archive::ArchiveReport {
            dry_run: false,
            planned: vec![result],
            succeeded: Vec::new(),
            rolled_back: Vec::new(),
            failed: vec![archive::ArchiveFailure {
                tasks_path: PathBuf::from("specs/fail/tasks.md"),
                operation: archive::ArchiveOperation::Publish,
                error: "bad\nwrite".to_string(),
            }],
        };

        let json = render_json(&report);
        assert!(json.contains("new\\nline"));
        assert!(json.contains("bad\\nwrite"));
        let parsed: serde_json::Value = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed["complete"], false);
        assert_eq!(parsed["partial"], false);
        assert_eq!(parsed["applied"], false);
        assert_eq!(parsed["failed"][0]["operation"], "publish");
    }

    #[test]
    fn text_paths_and_errors_visibly_escape_control_and_bidi_characters() {
        let report = archive::ArchiveReport {
            dry_run: false,
            planned: Vec::new(),
            succeeded: Vec::new(),
            rolled_back: Vec::new(),
            failed: vec![archive::ArchiveFailure {
                tasks_path: PathBuf::from("specs/new\n\u{202E}line/tasks.md"),
                operation: archive::ArchiveOperation::Read,
                error: "bad\u{0007}\nread".to_string(),
            }],
        };

        let text = render_text(&report);
        assert!(text.contains("new\\u{000A}\\u{202E}line"));
        assert!(text.contains("bad\\u{0007}\\u{000A}read"));
        assert!(!text.contains('\u{0007}'));
        assert!(!text.contains('\u{202E}'));
    }
}
