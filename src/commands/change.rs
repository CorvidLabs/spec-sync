use colored::Colorize;
use std::path::Path;
use std::process;

use crate::change::{self, ArtifactKind, ChangeKind, ChangeRecord, CreateChangeRequest};
use crate::cli::ChangeAction;
use crate::types::OutputFormat;

pub fn cmd_change(root: &Path, action: ChangeAction, format: OutputFormat) {
    let result = match action {
        ChangeAction::New {
            description,
            kind,
            specs,
            paths,
            artifacts,
            no_spec_change,
            rationale,
        } => ChangeKind::parse(&kind)
            .and_then(|kind| {
                change::create_change(
                    root,
                    CreateChangeRequest {
                        description,
                        kind,
                        affected_specs: specs,
                        affected_paths: paths,
                        requested_artifacts: artifacts
                            .iter()
                            .map(|value| ArtifactKind::parse(value))
                            .collect(),
                        no_spec_change,
                        rationale,
                    },
                )
            })
            .map(|record| print_record(root, &record, format, true)),
        ChangeAction::Answer {
            id,
            question,
            answer,
        } => change::answer_question(root, &id, &question, &answer)
            .map(|record| print_record(root, &record, format, true)),
        ChangeAction::Depend { id, on } => change::add_dependency(root, &id, &on)
            .map(|record| print_record(root, &record, format, false)),
        ChangeAction::List => {
            print_records(root, &change::list_changes(root), format);
            Ok(())
        }
        ChangeAction::Show { id } => {
            change::load_change(root, &id).map(|record| print_record(root, &record, format, true))
        }
        ChangeAction::Status { id } => {
            if let Some(id) = id {
                change::load_change(root, &id)
                    .map(|record| print_record(root, &record, format, false))
            } else {
                print_records(root, &change::list_changes(root), format);
                Ok(())
            }
        }
        ChangeAction::Approve { id, actor, note } => {
            change::approve_definition(root, &id, actor, note)
                .map(|record| print_transition(&record, format, "definition approved"))
        }
        ChangeAction::Start { id } => change::start_implementation(root, &id)
            .map(|record| print_transition(&record, format, "implementation started")),
        ChangeAction::Verify { id } => {
            change::verify_change(root, &id).map(|verification| match format {
                OutputFormat::Json => print_json(&verification),
                _ => println!(
                    "{} verification passed ({} command(s), {} requirement(s))",
                    "✓".green(),
                    verification.commands.len(),
                    verification.requirement_ids.len()
                ),
            })
        }
        ChangeAction::Reopen { id, actor, reason } => {
            change::reopen_change(root, &id, actor, reason).map(|result| match format {
                OutputFormat::Json => print_json(&result),
                _ => println!(
                    "{} {} reopened for fresh verification and closing approval",
                    "✓".green(),
                    result.change.id
                ),
            })
        }
        ChangeAction::Accept { id, actor, note } => change::accept_change(root, &id, actor, note)
            .map(|record| {
                print_transition(
                    &record,
                    format,
                    "closing approval recorded and deltas applied",
                )
            }),
        ChangeAction::Archive { id } => {
            change::archive_change(root, &id).map(|path| match format {
                OutputFormat::Json => print_json(&serde_json::json!({ "archived": path })),
                _ => println!("{} Archived to {}", "✓".green(), path.display()),
            })
        }
        ChangeAction::Check => {
            let report = change::check_project(root);
            let passed = report.errors.is_empty();
            match format {
                OutputFormat::Json => print_json(&report),
                _ => {
                    for warning in &report.warnings {
                        println!("{} {warning}", "warning:".yellow().bold());
                    }
                    for error in &report.errors {
                        eprintln!("{} {error}", "error:".red().bold());
                    }
                    if passed {
                        println!(
                            "{} {} active change(s) valid",
                            "✓".green(),
                            report.checked_changes
                        );
                    }
                }
            }
            if !passed {
                process::exit(1);
            }
            Ok(())
        }
        ChangeAction::Adopt { dry_run, source } => change::adopt(root, dry_run, source.as_deref())
            .map(|actions| match format {
                OutputFormat::Json => print_json(&serde_json::json!({
                    "dry_run": dry_run,
                    "actions": actions,
                })),
                _ => {
                    for action in actions {
                        println!("{} {action}", if dry_run { "○" } else { "✓" }.green());
                    }
                }
            }),
    };

    if let Err(error) = result {
        match format {
            OutputFormat::Json => print_json(&serde_json::json!({ "error": error })),
            _ => eprintln!("{} {error}", "error:".red().bold()),
        }
        process::exit(1);
    }
}

fn print_record(root: &Path, record: &ChangeRecord, format: OutputFormat, include_questions: bool) {
    let summary = change::summarize_change(root, record);
    let questions = if include_questions {
        change::next_questions(record)
    } else {
        Vec::new()
    };
    match format {
        OutputFormat::Json => print_json(&serde_json::json!({
            "change": record,
            "summary": summary,
            "questions": questions,
        })),
        _ => {
            println!("{} {}", record.id.bold(), record.title);
            println!("  State: {}", record.state.as_str());
            println!("  Next: {}", summary.next_action);
            if include_questions && !questions.is_empty() {
                println!("\nInterview:");
                for question in questions {
                    println!("  {} — {}", question.id.cyan(), question.prompt);
                    if !question.choices.is_empty() {
                        println!("    Choices: {}", question.choices.join(", "));
                    }
                }
            }
        }
    }
}

fn print_records(root: &Path, records: &[ChangeRecord], format: OutputFormat) {
    let summaries: Vec<_> = records
        .iter()
        .map(|record| change::summarize_change(root, record))
        .collect();
    match format {
        OutputFormat::Json => print_json(&summaries),
        _ if summaries.is_empty() => println!("No active SDD changes."),
        _ => {
            for summary in summaries {
                println!(
                    "{}  {:<13}  {}  next: {}",
                    summary.id.bold(),
                    summary.state.as_str(),
                    summary.title,
                    summary.next_action
                );
            }
        }
    }
}

fn print_transition(record: &ChangeRecord, format: OutputFormat, message: &str) {
    match format {
        OutputFormat::Json => print_json(&serde_json::json!({
            "id": record.id,
            "state": record.state,
            "message": message,
        })),
        _ => println!(
            "{} {}: {} ({})",
            "✓".green(),
            record.id,
            message,
            record.state.as_str()
        ),
    }
}

fn print_json<Value: serde::Serialize>(value: &Value) {
    match serde_json::to_string_pretty(value) {
        Ok(content) => println!("{content}"),
        Err(error) => eprintln!("{{\"error\":\"failed to serialize output: {error}\"}}"),
    }
}
