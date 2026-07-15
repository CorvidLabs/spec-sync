use colored::Colorize;
use std::path::Path;
use std::process;

use crate::change::{
    self, ArtifactKind, ChangeKind, ChangeRecord, CorrectionField, CreateChangeRequest,
};
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
            .and_then(|record| print_record(root, &record, format, true)),
        ChangeAction::Answer {
            id,
            question,
            answer,
        } => change::answer_question(root, &id, &question, &answer)
            .and_then(|record| print_record(root, &record, format, true)),
        ChangeAction::Depend { id, on } => change::add_dependency(root, &id, &on)
            .and_then(|record| print_record(root, &record, format, false)),
        ChangeAction::Supersede {
            id,
            predecessor,
            path,
            module,
            digest,
        } => change::add_supersedes_obligation(root, &id, &predecessor, &path, &module, &digest)
            .and_then(|record| print_record(root, &record, format, false)),
        ChangeAction::List => {
            print_records(root, &change::list_changes(root), format);
            Ok(())
        }
        ChangeAction::Show { id } => change::load_change(root, &id)
            .and_then(|record| print_record(root, &record, format, true)),
        ChangeAction::Status { id } => {
            if let Some(id) = id {
                change::load_change(root, &id)
                    .and_then(|record| print_record(root, &record, format, false))
            } else {
                print_records(root, &change::list_changes(root), format);
                Ok(())
            }
        }
        ChangeAction::Approve {
            id,
            actor,
            note,
            portable_5_0_1,
        } => {
            let result = if portable_5_0_1 {
                change::approve_definition_portable_v501(root, &id, actor, note)
            } else {
                change::approve_definition(root, &id, actor, note)
            };
            result.map(|record| print_transition(&record, format, "definition approved"))
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
        ChangeAction::Correct {
            id,
            field,
            value,
            actor,
            reason,
        } => CorrectionField::parse(&field).and_then(|field| {
            change::correct_interview_metadata(root, &id, field, value, actor, reason).map(
                |result| match format {
                    OutputFormat::Json => print_json(&result),
                    _ => {
                        println!(
                            "{} {} corrected {} from {} to {} as {}",
                            "✓".green(),
                            result.change.id,
                            result.correction.field.as_str(),
                            result.correction.prior_effective_value,
                            result.correction.corrected_value,
                            result.correction.actor
                        );
                        if !result.correction.added_artifacts.is_empty() {
                            println!(
                                "  Added artifacts: {}",
                                result
                                    .correction
                                    .added_artifacts
                                    .iter()
                                    .map(ArtifactKind::file_name)
                                    .collect::<Vec<_>>()
                                    .join(", ")
                            );
                        }
                        println!("  Next: {}", result.summary.next_action);
                    }
                },
            )
        }),
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
                    for result in &report.terminal_evidence {
                        println!(
                            "{} evidence: {}",
                            result.id,
                            result.evidence.validity.as_str()
                        );
                        if let Some(reason) = &result.evidence.reason {
                            println!("  reason: {reason}");
                        }
                    }
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

fn print_record(
    root: &Path,
    record: &ChangeRecord,
    format: OutputFormat,
    include_questions: bool,
) -> Result<(), String> {
    let summary = change::summarize_change(root, record);
    let effective_definition = change::effective_change_definition(root, record)?;
    let corrections = change::correction_history(root, record)?;
    let questions = if include_questions {
        change::next_questions(record)
    } else {
        Vec::new()
    };
    match format {
        OutputFormat::Json => print_json(&serde_json::json!({
            "change": record,
            "effective_definition": effective_definition,
            "corrections": corrections,
            "summary": summary,
            "questions": questions,
        })),
        _ => {
            println!("{} {}", record.id.bold(), record.title);
            println!("  State: {}", record.state.as_str());
            println!("  Next: {}", summary.next_action);
            if !corrections.is_empty() {
                println!("  Corrections:");
                for correction in &corrections {
                    println!(
                        "    {}: {} → {} by {} at {} — {}",
                        correction.field.as_str(),
                        correction.prior_effective_value,
                        correction.corrected_value,
                        correction.actor,
                        correction.timestamp,
                        correction.reason
                    );
                    println!(
                        "      digests: {} → {}",
                        correction.prior_view_digest, correction.corrected_view_digest
                    );
                }
                println!(
                    "  Effective answers: {}",
                    effective_definition
                        .answers
                        .iter()
                        .map(|(field, value)| format!("{field}={value}"))
                        .collect::<Vec<_>>()
                        .join(", ")
                );
            }
            if let Some(evidence) = &summary.terminal_evidence {
                println!("  Evidence: {}", evidence.validity.as_str());
                if let Some(reason) = &evidence.reason {
                    println!("  Evidence reason: {reason}");
                }
            }
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
    Ok(())
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
                let evidence = summary
                    .terminal_evidence
                    .as_ref()
                    .map(|evidence| format!("  evidence: {}", evidence.validity.as_str()))
                    .unwrap_or_default();
                println!(
                    "{}  {:<13}  {}  next: {}{}",
                    summary.id.bold(),
                    summary.state.as_str(),
                    summary.title,
                    summary.next_action,
                    evidence
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
