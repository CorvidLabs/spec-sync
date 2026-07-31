use colored::Colorize;
use std::fs;
use std::path::{Path, PathBuf};
use std::process;

use crate::change::{
    self, ArtifactKind, ChangeKind, ChangeRecord, ChangeState, CorrectionField,
    CreateChangeRequest, InterviewQuestion,
};
use crate::cli::ChangeAction;
use crate::types::OutputFormat;

pub fn cmd_change(root: &Path, action: ChangeAction, format: OutputFormat, strict: bool) {
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
            .and_then(|record| print_record(root, &record, format, true, strict)),
        ChangeAction::Answer {
            id,
            question,
            answer,
        } => change::answer_question(root, &id, &question, &answer)
            .and_then(|record| print_record(root, &record, format, true, strict)),
        ChangeAction::Depend { id, on } => change::add_dependency(root, &id, &on)
            .and_then(|record| print_record(root, &record, format, false, strict)),
        ChangeAction::Supersede {
            id,
            predecessor,
            path,
            module,
            digest,
        } => change::add_supersedes_obligation(root, &id, &predecessor, &path, &module, &digest)
            .and_then(|record| print_record(root, &record, format, false, strict)),
        ChangeAction::List => {
            let _scope = change::begin_change_read_scope(root);
            print_records(root, &change::list_changes(root), format, strict);
            Ok(())
        }
        ChangeAction::Show { id } => {
            let _scope = change::begin_change_read_scope(root);
            change::load_change(root, &id)
                .and_then(|record| print_record(root, &record, format, true, strict))
        }
        ChangeAction::Status { id } => {
            let _scope = change::begin_change_read_scope(root);
            if let Some(id) = id {
                change::load_change(root, &id)
                    .and_then(|record| print_record(root, &record, format, false, strict))
            } else {
                print_records(root, &change::list_changes(root), format, strict);
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
            let result = if strict {
                change::verify_change_with_strict(root, &id, true)
            } else {
                change::verify_change(root, &id)
            };
            result.map(|verification| match format {
                OutputFormat::Json => print_json(&verification),
                _ => println!(
                    "{} verification passed ({} command(s), {} requirement(s))",
                    "✓".green(),
                    verification.commands.len(),
                    verification.requirement_ids.len()
                ),
            })
        }
        ChangeAction::Review {
            id,
            reviewer,
            verdict,
        } => change::ScopedReviewVerdict::parse(&verdict).and_then(|verdict| {
            let result = if verdict == change::ScopedReviewVerdict::Pass {
                change::record_scoped_review(root, &id, reviewer)
            } else {
                change::record_scoped_review_with_verdict(root, &id, reviewer, verdict)
            };
            result.map(|review| {
                match format {
                    OutputFormat::Json => print_json(&review),
                    _ => println!(
                        "{} {} independent review recorded as {} at {}",
                        "✓".green(),
                        review.change_id,
                        review.verdict.as_str(),
                        review.implementation_commit
                    ),
                }
            })
        }),
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
        ChangeAction::Accept { id, actor, note } => change::load_change(root, &id)
            .and_then(|record| {
                if record.workflow_version >= 2 {
                    return Err(format!(
                        "{} uses the single 6.0 workflow; record scoped review and run `specsync change finalize {}`",
                        record.id, record.id
                    ));
                }
                change::accept_change(root, &id, actor, note)
            })
            .map(|record| {
                print_transition(
                    &record,
                    format,
                    "closing approval recorded and deltas applied",
                )
            }),
        ChangeAction::Archive { id } => {
            change::load_change(root, &id).and_then(|record| {
                if record.workflow_version >= 2 {
                    return Err(format!(
                        "{} uses same-PR finalization; run `specsync change finalize {}`",
                        record.id, record.id
                    ));
                }
                change::archive_change(root, &id)
            }).map(|path| match format {
                OutputFormat::Json => print_json(&serde_json::json!({ "archived": path })),
                _ => println!("{} Archived to {}", "✓".green(), path.display()),
            })
        }
        ChangeAction::Finalize { id } => change::finalize_change(root, &id).and_then(|path| {
            let finalization_path = path.join("finalization.json");
            let content = fs::read_to_string(&finalization_path).map_err(|error| {
                format!("failed to read {}: {error}", finalization_path.display())
            })?;
            let finalization: change::FinalizationRecord =
                serde_json::from_str(&content).map_err(|error| {
                    format!(
                        "invalid finalization {}: {error}",
                        finalization_path.display()
                    )
                })?;
            match format {
                OutputFormat::Json => print_json(&serde_json::json!({
                    "id": id,
                    "archived": path,
                    "implementation_commit": finalization.implementation_commit,
                    "implementation_tree": finalization.implementation_tree,
                    "contract_digest": finalization.contract_digest,
                    "workspace_digest": finalization.workspace_digest,
                    "review_digest": finalization.review_digest,
                    "finalization_digest": finalization.finalization_digest,
                    "ready_for_github_merge": true,
                    "next_action": "merge the PR on GitHub",
                })),
                _ => {
                    // Digests stay in --json only; text mode names the archive + commit.
                    println!("{} {} finalized on this PR", "✓".green(), id);
                    println!("  Archive: {}", path.display());
                    println!("  Implementation: {}", finalization.implementation_commit);
                    println!("  Next: merge the PR on GitHub");
                }
            }
            Ok(())
        }),
        ChangeAction::Check { id } => {
            let verification = if strict {
                change::check_change_with_strict(root, id.as_deref(), true)
            } else {
                change::check_change(root, id.as_deref())
            };
            let report = change::check_project(root);
            let passed = verification.is_ok() && report.errors.is_empty();
            match format {
                OutputFormat::Json => print_json(&serde_json::json!({
                    "verification": verification.as_ref().ok(),
                    "report": report,
                })),
                _ => {
                    if let Err(error) = &verification {
                        eprintln!("{} {error}", "error:".red().bold());
                    }
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
                            "{} {} active change(s) checked",
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
    strict: bool,
) -> Result<(), String> {
    let questions = if include_questions {
        change::next_questions(record)
    } else {
        Vec::new()
    };
    match format {
        OutputFormat::Json => {
            // JSON keeps digests and correction ledgers for machine consumers.
            let summary = change::summarize_change_with_strict(root, record, strict);
            let effective_definition = change::effective_change_definition(root, record)?;
            let corrections = change::correction_history(root, record)?;
            print_json(&serde_json::json!({
                "change": record,
                "effective_definition": effective_definition,
                "corrections": corrections,
                "summary": summary,
                "questions": questions,
            }));
        }
        _ => {
            // Text mode must not invoke digest-bearing loaders into cleartext sinks
            // (CodeQL rust/cleartext-logging). Human output uses interview/state only.
            let id = record.id.clone();
            let title = record.title.clone();
            let state = record.state.as_str().to_owned();
            println!("{} {}", id.bold(), title);
            println!("  State: {state}");
            println!("  Next: {}", text_mode_next_action(record, &questions));
            if !record.answers.is_empty() {
                let answer_summary = record
                    .answers
                    .iter()
                    .map(|(field, value)| format!("{field}={value}"))
                    .collect::<Vec<_>>()
                    .join(", ");
                println!("  Answers: {answer_summary}");
            }
            if record.correction_count > 0 {
                println!(
                    "  Corrections: {} recorded (use --json for the audit ledger)",
                    record.correction_count
                );
            }
            if !record.acceptance_owner_corrections.is_empty() {
                println!(
                    "  Acceptance owner corrections: {} (use --json for details)",
                    record.acceptance_owner_corrections.len()
                );
            }
            if include_questions && !questions.is_empty() {
                println!("\nInterview:");
                for question in &questions {
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

/// Human next-step string from interview/state only (no digest-bearing loaders).
fn text_mode_next_action(record: &ChangeRecord, questions: &[InterviewQuestion]) -> String {
    let id = record.id.as_str();
    match record.state {
        ChangeState::Draft if questions.is_empty() => {
            format!("run `specsync change approve {id} --actor <name>`")
        }
        ChangeState::Draft => {
            let question = questions
                .first()
                .map(|question| question.id.as_str())
                .unwrap_or("<question>");
            format!("run `specsync change answer {id} {question} <answer>`")
        }
        ChangeState::Approved | ChangeState::Implementing => {
            format!("run `specsync change check {id}`")
        }
        ChangeState::Verifying => {
            format!("run `specsync change check {id}` or finalize when ready")
        }
        ChangeState::Accepted if record.workflow_version >= 2 => {
            format!("run `specsync change finalize {id}`")
        }
        ChangeState::Accepted => format!("run `specsync change archive {id}`"),
        ChangeState::Archived => "no further action".into(),
    }
}

fn print_records(root: &Path, records: &[ChangeRecord], format: OutputFormat, strict: bool) {
    match format {
        OutputFormat::Json => {
            let summaries: Vec<_> = records
                .iter()
                .map(|record| change::summarize_change_with_strict(root, record, strict))
                .collect();
            print_json(&summaries);
        }
        _ if records.is_empty() => println!("No active SDD changes."),
        _ => {
            // Text list view avoids digest-bearing summarize loaders (cleartext-logging).
            for record in records {
                let questions = change::next_questions(record);
                let id = record.id.clone();
                let title = record.title.clone();
                let state = record.state.as_str().to_owned();
                let next = text_mode_next_action(record, &questions);
                println!("{:<14}  {state:<13}  {title}  next: {next}", id.bold());
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
