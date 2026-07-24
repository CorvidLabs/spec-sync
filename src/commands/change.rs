use colored::Colorize;
use std::fs;
use std::path::{Path, PathBuf};
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
        } => {
            let digest = match digest {
                Some(digest) => Ok(digest),
                None => change::resolve_predecessor_entry_digest(root, &predecessor, &path),
            };
            digest.and_then(|digest| {
                change::add_supersedes_obligation(root, &id, &predecessor, &path, &module, &digest)
                    .and_then(|record| print_record(root, &record, format, false))
            })
        }
        ChangeAction::List => {
            let listing = change::list_changes_with_errors(root);
            print_records(root, &listing.records, format);
            report_listing_errors(&listing)
        }
        ChangeAction::Show { id } => change::load_change(root, &id)
            .and_then(|record| print_record(root, &record, format, true)),
        ChangeAction::Status { id } => {
            if let Some(id) = id {
                change::load_change(root, &id)
                    .and_then(|record| print_record(root, &record, format, false))
            } else {
                let listing = change::list_changes_with_errors(root);
                print_records(root, &listing.records, format);
                report_listing_errors(&listing)
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
        ChangeAction::CorrectOwner {
            id,
            paths,
            modules,
            manifest,
            all_missing,
            actor,
            reason,
        } => {
            let result = if all_missing {
                if !paths.is_empty() || manifest.is_some() {
                    Err(
                        "correct-owner accepts only one of --path, --manifest, or --all-missing"
                            .into(),
                    )
                } else if modules.len() != 1 {
                    Err("--all-missing requires exactly one --spec module".into())
                } else {
                    let before = change::load_change(root, &id)
                        .map(|record| record.acceptance_owner_corrections.len())
                        .unwrap_or(0);
                    change::add_missing_acceptance_owner_corrections(
                        root,
                        &id,
                        modules[0].clone(),
                        actor,
                        reason,
                    )
                    .map(|record| {
                        let appended = record
                            .acceptance_owner_corrections
                            .len()
                            .saturating_sub(before);
                        (record, appended)
                    })
                }
            } else {
                resolve_correct_owner_entries(paths, modules, manifest).and_then(|entries| {
                    let appended = entries.len();
                    change::add_acceptance_owner_corrections(root, &id, entries, actor, reason)
                        .map(|record| (record, appended))
                })
            };
            result.map(|(record, appended)| match format {
                OutputFormat::Json => print_json(&record),
                _ => {
                    if appended == 1 {
                        if let Some(correction) = record.acceptance_owner_corrections.last() {
                            println!(
                                "{} {} corrected owner {} for {} as {}",
                                "✓".green(),
                                record.id,
                                correction.module,
                                correction.path,
                                correction.actor
                            );
                        }
                    } else {
                        let actor = record
                            .acceptance_owner_corrections
                            .last()
                            .map(|correction| correction.actor.as_str())
                            .unwrap_or("unknown");
                        println!(
                            "{} {} corrected {} acceptance owners as {}",
                            "✓".green(),
                            record.id,
                            appended,
                            actor
                        );
                    }
                    println!("  Next: approve");
                }
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
            if !record.dependencies.is_empty() {
                println!("  Dependencies: {}", record.dependencies.join(", "));
            }
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
            if !record.acceptance_owner_corrections.is_empty() {
                println!("  Acceptance owner corrections:");
                for correction in &record.acceptance_owner_corrections {
                    println!(
                        "    {}: {} owned by {} by {} at {} — {}",
                        correction.sequence,
                        correction.path,
                        correction.module,
                        correction.actor,
                        correction.timestamp,
                        correction.reason
                    );
                }
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
    if summary
        .terminal_evidence
        .as_ref()
        .is_some_and(|evidence| {
            evidence.validity == change::TerminalEvidenceValidity::CorruptHistory
        })
    {
        return Err(format!(
            "{} has corrupt archived evidence; inspect the evidence reason above and restore the committed evidence or reopen the change",
            record.id
        ));
    }
    Ok(())
}

/// Surface one error row per malformed workspace and fail the command so a
/// corrupt change is never indistinguishable from an empty project.
fn report_listing_errors(listing: &change::ChangeListing) -> Result<(), String> {
    for error in &listing.errors {
        eprintln!("error: {error}");
    }
    if listing.errors.is_empty() {
        Ok(())
    } else {
        Err(format!(
            "{} change workspace(s) are unreadable; repair or remove them and retry",
            listing.errors.len()
        ))
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

fn resolve_correct_owner_entries(
    paths: Vec<String>,
    modules: Vec<String>,
    manifest: Option<PathBuf>,
) -> Result<Vec<(String, String)>, String> {
    if paths.is_empty() && manifest.is_none() {
        return Err(
            "correct-owner requires --path, --manifest, or --all-missing with --spec".into(),
        );
    }
    if !paths.is_empty() && manifest.is_some() {
        return Err(
            "correct-owner accepts only one of --path, --manifest, or --all-missing".into(),
        );
    }
    if let Some(manifest) = manifest {
        if !modules.is_empty() {
            return Err(
                "--manifest cannot be combined with --spec; encode modules in the manifest".into(),
            );
        }
        return parse_owner_manifest(&manifest);
    }
    if modules.is_empty() {
        return Err("correct-owner --path requires at least one --spec".into());
    }
    if modules.len() == 1 {
        let module = modules[0].clone();
        return Ok(paths
            .into_iter()
            .map(|path| (path, module.clone()))
            .collect());
    }
    if modules.len() != paths.len() {
        return Err(format!(
            "correct-owner received {} --path values and {} --spec values; provide one --spec for all paths or matching pairs",
            paths.len(),
            modules.len()
        ));
    }
    Ok(paths.into_iter().zip(modules).collect())
}

fn parse_owner_manifest(path: &Path) -> Result<Vec<(String, String)>, String> {
    let content = fs::read_to_string(path).map_err(|error| {
        format!(
            "failed to read correct-owner manifest {}: {error}",
            path.display()
        )
    })?;
    let trimmed = content.trim();
    if trimmed.is_empty() {
        return Err("correct-owner manifest is empty".into());
    }
    if trimmed.starts_with('[') {
        let entries: Vec<serde_json::Value> = serde_json::from_str(trimmed)
            .map_err(|error| format!("invalid correct-owner JSON manifest: {error}"))?;
        let mut resolved = Vec::with_capacity(entries.len());
        for (index, entry) in entries.iter().enumerate() {
            let path = entry
                .get("path")
                .and_then(|value| value.as_str())
                .ok_or_else(|| {
                    format!("correct-owner manifest entry {} is missing path", index + 1)
                })?;
            let module = entry
                .get("module")
                .and_then(|value| value.as_str())
                .ok_or_else(|| {
                    format!(
                        "correct-owner manifest entry {} is missing module",
                        index + 1
                    )
                })?;
            resolved.push((path.to_string(), module.to_string()));
        }
        if resolved.is_empty() {
            return Err("correct-owner JSON manifest contains no entries".into());
        }
        return Ok(resolved);
    }
    let mut resolved = Vec::new();
    for (index, line) in content.lines().enumerate() {
        let line = line.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        let mut parts = line.splitn(2, '\t');
        let path = parts
            .next()
            .map(str::trim)
            .filter(|value| !value.is_empty())
            .ok_or_else(|| {
                format!(
                    "correct-owner TSV manifest line {} is missing path",
                    index + 1
                )
            })?;
        let module = parts
            .next()
            .map(str::trim)
            .filter(|value| !value.is_empty())
            .ok_or_else(|| {
                format!(
                    "correct-owner TSV manifest line {} is missing module",
                    index + 1
                )
            })?;
        resolved.push((path.to_string(), module.to_string()));
    }
    if resolved.is_empty() {
        return Err("correct-owner TSV manifest contains no entries".into());
    }
    Ok(resolved)
}
