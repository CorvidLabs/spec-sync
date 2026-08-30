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

const INVALID_CORRECTION_LEDGER_TEXT: &str = "correction ledger integrity is invalid; restore corrections.json from trusted history before inspecting lifecycle status";

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
            .and_then(|record| {
                if !matches!(format, OutputFormat::Json) {
                    print_accumulated_lessons(root, &record);
                }
                print_record(root, &record, format, true, strict)
            }),
        ChangeAction::Answer {
            id,
            question,
            answer,
        } => change::answer_question_with_snapshot(root, &id, &question, &answer)
            .and_then(|result| print_mutation_record(root, &id, &result, format, true, strict)),
        ChangeAction::Depend { id, on } => change::add_dependency_with_snapshot(root, &id, &on)
            .and_then(|result| print_mutation_record(root, &id, &result, format, false, strict)),
        ChangeAction::Supersede {
            id,
            predecessor,
            path,
            module,
            digest,
        } => change::add_supersedes_obligation_with_snapshot(
            root,
            &id,
            &predecessor,
            &path,
            &module,
            &digest,
        )
        .and_then(|result| print_mutation_record(root, &id, &result, format, false, strict)),
        ChangeAction::List => {
            let _scope = change::begin_change_read_scope(root);
            change::list_changes(root)
                .and_then(|roster| print_roster(root, &roster, format, strict))
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
                change::list_changes(root)
                    .and_then(|roster| print_roster(root, &roster, format, strict))
            }
        }
        ChangeAction::ShipStatus { id } => {
            let _scope = change::begin_change_read_scope(root);
            if let Some(id) = id {
                change::load_change(root, &id)
                    .and_then(|record| print_ship_status(root, &record, format))
            } else {
                change::list_changes(root).and_then(|roster| {
                let records = &roster.records;
                if records.is_empty() && !roster.is_degraded() {
                    match format {
                        OutputFormat::Json => print_json(&serde_json::json!({ "changes": [] })),
                        _ => println!("No active SDD changes."),
                    }
                    Ok(())
                } else if matches!(format, OutputFormat::Json) {
                    records
                        .iter()
                        .map(|record| ship_status_report(root, record))
                        .collect::<Result<Vec<_>, _>>()
                        .map(|reports| {
                            let error = unreadable_error(&roster).err();
                            let degraded = roster.is_degraded();
                            print_json(&serde_json::json!({
                                "changes": reports,
                                "unreadable": unreadable_json(&roster),
                                "error": error,
                            }));
                            // See print_roster: returning Err here would append a
                            // second JSON document to stdout.
                            if degraded {
                                process::exit(1);
                            }
                        })
                } else {
                    records
                        .iter()
                        .try_for_each(|record| {
                            print_ship_status(root, record, format)?;
                            println!();
                            Ok(())
                        })
                        .and_then(|()| {
                            print_unreadable_rows(&roster);
                            unreadable_error(&roster)
                        })
                }
                })
            }
        }
        ChangeAction::Ship {
            id,
            dry_run,
            push,
            wait,
            wait_timeout_secs,
        } => run_ship(
            root,
            id.as_deref(),
            dry_run,
            push,
            wait,
            wait_timeout_secs,
            format,
        ),
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
                    "lesson_bundle": path.join(change::LESSON_BUNDLE_FILE),
                    "next_action": lessons_next_action(root, &id, &path),
                })),
                _ => {
                    // Digests stay in --json only; text mode names the archive + commit.
                    println!("{} {} finalized on this PR", "✓".green(), id);
                    println!("  Archive: {}", path.display());
                    println!("  Implementation: {}", finalization.implementation_commit);
                    println!("  Next: {}", lessons_next_action(root, &id, &path));
                }
            }
            Ok(())
        }),
        ChangeAction::Check { id, commit, push } if commit || push => {
            if push && !commit {
                Err("--push requires --commit".to_string())
            } else {
                run_checked_commit(root, id.as_deref(), strict, push, format)
            }
        }
        ChangeAction::Check { id, .. } => {
            // Scoped verification only — project integrity is `change audit`.
            let display_id = id.clone().unwrap_or_else(|| "open change".into());
            if !matches!(format, OutputFormat::Json) {
                println!("Checking {display_id}…");
            }
            let verification = if strict {
                change::check_change_with_strict(root, id.as_deref(), true)
            } else {
                change::check_change(root, id.as_deref())
            };
            // A failed check is the moment a lesson exists: an approach was tried and did not
            // work. Say so while it is still true, and name where it goes.
            //
            // Verification failures surface as an Err from `verify_change`, NOT as a record with
            // `passed: false` — a first attempt at this hint sat in that branch and never fired
            // for the common case. Measured, not assumed.
            //
            // Only on failure, deliberately. Nudging on every green check would be noise, and
            // there is usually nothing to record.
            if verification.is_err()
                && !matches!(format, OutputFormat::Json)
                && let Some(change_id) = id
                    .clone()
                    .or_else(|| change::active_change_id(root))
                    .as_deref()
            {
                eprintln!(
                    "  {} if this failure taught you something, record it in .specsync/changes/{}/context.md while it is fresh — dead ends are what the next change to this module needs, and `finalize` folds them into the spec",
                    "Lesson:".bold(),
                    change_id
                );
            }
            match format {
                OutputFormat::Json => match &verification {
                    Ok(Some(record)) => {
                        print_json(&serde_json::json!({ "verification": record }));
                        if !record.passed {
                            process::exit(1);
                        }
                    }
                    Ok(None) => {
                        print_json(&serde_json::json!({
                            "verification": null,
                            "message": "nothing to check",
                        }));
                    }
                    Err(error) => {
                        print_json(&serde_json::json!({ "error": error }));
                        process::exit(1);
                    }
                },
                _ => match &verification {
                    Ok(Some(record)) => {
                        // Prefer the ID from the change workspace after verify.
                        let verified_id =
                            id.clone().or_else(|| change::active_change_id(root));
                        let label = verified_id.as_deref().unwrap_or(display_id.as_str());
                        if record.passed {
                            println!("{} {} verified", "✓".green(), label);
                            if let Some(change_id) = &verified_id
                                && let Ok(change_record) = change::load_change(root, change_id)
                            {
                                let questions = change::next_questions(&change_record);
                                println!(
                                    "  Next: {}",
                                    text_mode_next_action(root, &change_record, &questions)
                                );
                            }
                        } else {
                            eprintln!(
                                "{} {} verification failed",
                                "error:".red().bold(),
                                label
                            );
                            for command in &record.commands {
                                if !command.success {
                                    eprintln!(
                                        "  failed: {} (exit {:?})",
                                        command.command, command.exit_code
                                    );
                                }
                            }
                            println!(
                                "  Next: fix verification failures and re-run `specsync change check{}`",
                                verified_id
                                    .as_ref()
                                    .map(|value| format!(" {value}"))
                                    .unwrap_or_default()
                            );
                            process::exit(1);
                        }
                    }
                    Ok(None) => {
                        println!("Nothing to check (no approved/implementing/verifying change).");
                    }
                    Err(error) => {
                        eprintln!("{} {error}", "error:".red().bold());
                        process::exit(1);
                    }
                },
            }
            Ok(())
        }
        ChangeAction::Audit => {
            let report = change::audit_project(root);
            let passed = report.errors.is_empty();
            match format {
                OutputFormat::Json => {
                    print_json(&serde_json::json!({ "report": report }));
                }
                _ => {
                    if report.enabled {
                        println!(
                            "Auditing active changes ({})…",
                            report.checked_changes
                        );
                    }
                    for warning in &report.warnings {
                        println!("{} {warning}", "warning:".yellow().bold());
                    }
                    for error in &report.errors {
                        eprintln!("{} {error}", "error:".red().bold());
                    }
                    if passed {
                        println!(
                            "{} audit passed ({} active)",
                            "✓".green(),
                            report.checked_changes
                        );
                    } else {
                        println!("  Next: fix active workspace / living-spec issues above");
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
            // `change supersede --digest` needs a specsync.acceptance-entry.v1
            // digest and nothing emitted one, so callers had to read
            // verification.json by hand. Surface the entries here.
            let acceptance_entries = change::acceptance_entries(root, record);
            print_json(&serde_json::json!({
                "change": record,
                "effective_definition": effective_definition,
                "acceptance_entries": acceptance_entries,
                "corrections": corrections,
                "summary": summary,
                "questions": questions,
            }));
        }
        _ => {
            // Text mode must not invoke digest-bearing loaders into cleartext sinks
            // (CodeQL rust/cleartext-logging / alert #58). Human output uses interview
            // identity/state only; digests and correction-ledger details stay JSON-only.
            ensure_text_correction_ledger_valid(root, record)?;
            // `text_mode_next_action` uses lightweight artifact file reads only.
            print_change_text_identity(record);
            let next = text_mode_next_action(root, record, &questions);
            println!("  Next: {next}");
            print_change_text_answers(record);
            print_change_text_correction_counts(record);
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

fn print_mutation_record(
    root: &Path,
    id: &str,
    result: &change::DefinitionMutationResult,
    format: OutputFormat,
    include_questions: bool,
    strict: bool,
) -> Result<(), String> {
    let record = &result.change;
    let questions = if include_questions {
        change::next_questions(record)
    } else {
        Vec::new()
    };
    match format {
        OutputFormat::Json => {
            // The summary was built while the domain mutation still held the project lock.
            // Reusing it keeps the machine response consistent with the validated correction
            // snapshot even if corrections.json changes after persistence.
            let summary = if strict {
                &result.strict_summary
            } else {
                &result.summary
            };
            let acceptance_entries = change::acceptance_entries(root, record);
            print_json(&serde_json::json!({
                "change": record,
                "effective_definition": result.effective_definition,
                "acceptance_entries": acceptance_entries,
                "corrections": result.corrections,
                "summary": summary,
                "questions": questions,
            }));
        }
        _ => {
            // The domain operation validated correction history while holding the mutation lock.
            // Do not turn successful persistence into a false command failure by rereading that
            // ledger after the transaction has completed.
            print_change_text_identity(record);
            let next = text_mode_next_action(root, record, &questions);
            println!("  Next: {next}");
            print_change_text_answers(record);
            // Keep correction-derived snapshot values out of the human sink. A state-only reload
            // preserves the established counts without rereading corrections.json; failure is
            // intentionally non-fatal because the mutation has already persisted successfully.
            if let Ok(current) = change::load_change(root, id) {
                print_change_text_correction_counts(&current);
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

/// Print only non-sensitive identity fields for human text sinks.
///
/// Digests / correction ledgers must not flow into `println!` (CodeQL
/// `rust/cleartext-logging`). Callers must not pass values produced by
/// `effective_change_definition` / `correction_history` into this path.
fn print_change_text_identity(record: &ChangeRecord) {
    let id = record.id.as_str();
    let title = record.title.as_str();
    let state = record.state.as_str();
    println!("{} {}", id.bold(), title);
    println!("  State: {state}");
    print_legacy_workflow_notice(record);
}

/// Name the lifecycle when — and only when — it is the weaker one.
///
/// A repository upgraded from 5.x keeps `version: 1` in its policy, `init` short-circuits on an
/// existing project without raising it, and every change created there is workflow-v1. Nothing said
/// so until `ship` refused several verbs later, by which point the change cannot be re-created on
/// the other lifecycle without redoing the work.
///
/// This announces STATE, not a verb. A reader who believes they are on v2 needs their assumption
/// contradicted; being told a command exists does not contradict anything, because they had no
/// reason to think it applied to them. v2 stays silent so the normal path gains no noise.
fn print_legacy_workflow_notice(record: &ChangeRecord) {
    if record.workflow_version >= 2 {
        return;
    }
    println!(
        "  {} workflow v1 (legacy) — this change uses `change accept` and `change archive`, not `change finalize`",
        "!".yellow().bold()
    );
    println!("    adopt the current lifecycle for NEW changes with `specsync change adopt`");
}

/// Print only non-sensitive correction counts from state.json for human text sinks.
///
/// Callers must source this record independently from correction-ledger validation results so
/// correction values, ledger bytes, and digest material cannot flow into cleartext output.
fn print_change_text_correction_counts(record: &ChangeRecord) {
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
}

fn print_change_text_answers(record: &ChangeRecord) {
    if record.answers.is_empty() {
        return;
    }
    let answer_summary = record
        .answers
        .iter()
        .map(|(field, value)| format!("{field}={value}"))
        .collect::<Vec<_>>()
        .join(", ");
    println!("  Answers: {answer_summary}");
}

/// Human next-step string from interview/state and lightweight artifact completeness.
/// Avoids digest-bearing loaders (CodeQL cleartext-logging); only reads selected artifact files.
fn text_mode_next_action(
    root: &Path,
    record: &ChangeRecord,
    questions: &[InterviewQuestion],
) -> String {
    let id = record.id.as_str();
    match record.state {
        ChangeState::Draft if !questions.is_empty() => {
            let question = questions
                .first()
                .map(|question| question.id.as_str())
                .unwrap_or("<question>");
            format!("run `specsync change answer {id} {question} <answer>`")
        }
        ChangeState::Draft if !change::artifacts_complete_for_guidance(root, record) => {
            format!("complete selected artifacts, then run `specsync change status {id}`")
        }
        ChangeState::Draft => {
            format!("run `specsync change approve {id} --actor <name>`")
        }
        ChangeState::Approved | ChangeState::Implementing => {
            format!("run `specsync change check {id}`")
        }
        ChangeState::Verifying => {
            format!(
                "run `specsync change check {id} --commit` if needed, then `specsync change ship-status {id}` (or `ship {id}`) — independent review then finalize before merging; merging first orphans verification evidence and blocks earlier accepted changes sharing a delivery input from archiving"
            )
        }
        ChangeState::Accepted if record.workflow_version >= 2 => {
            format!("run `specsync change ship {id}` or `specsync change finalize {id}`")
        }
        ChangeState::Accepted => format!("run `specsync change archive {id}`"),
        ChangeState::Archived => "no further action".into(),
    }
}

fn ensure_text_correction_ledger_valid(root: &Path, record: &ChangeRecord) -> Result<(), String> {
    change::effective_change_definition(root, record)
        .is_ok()
        .then_some(())
        .ok_or_else(|| INVALID_CORRECTION_LEDGER_TEXT.to_string())
}

/// Ship readiness for one change: HEAD tip class, verification tip health, review,
/// trust guidance for staged product → review → archive tips, and the merge-before-finalize trap.
///
/// When `GITHUB_TOKEN` is set and the git remote is a GitHub repo, queries check-runs for the
/// parent (or tip) SHA. Offline / no-token stays on `local_guidance`. Force offline with
/// `SPECSYNC_SHIP_LOCAL_GUIDANCE=1`.
fn ship_status_report(root: &Path, record: &ChangeRecord) -> Result<serde_json::Value, String> {
    let questions = change::next_questions(record);
    let lifecycle_next = text_mode_next_action(root, record, &questions);
    let tip = classify_head_tip(root)?;

    // Resolve the workspace wherever the change actually lives. Both reads below used
    // to hard-code `.specsync/changes/<id>/`, a parallel implementation of `change_dir`
    // that an archived change has moved out of — so a finalized change reported
    // `Verification: none` / `Review: missing` for evidence sitting in its archive
    // package (#534). `find_change_dir` already answers active-or-archive and is the
    // primitive to reuse; adding a third resolver beside it is how this class recurs.
    //
    // Falling back to the active path keeps this read-only helper total: resolution can
    // fail on an ambiguous or malformed id, and a status command must still render.
    let evidence_dir = change::find_change_dir(root, &record.id)
        .unwrap_or_else(|_| root.join(".specsync/changes").join(&record.id));

    let verification_path = evidence_dir.join("verification.json");
    let (verification_commit, verification_present, verification_ancestor) =
        if verification_path.is_file() {
            // Lenient by design. A strict `?` here turns `ship-status` and `ship` from rc=0
            // into rc=1 on a repository whose archived evidence is already damaged — the fix
            // for an inspection command must not be the thing that bricks inspection. An
            // unreadable or unparseable artifact degrades to "no evidence recorded".
            let value = fs::read_to_string(&verification_path)
                .ok()
                .and_then(|raw| serde_json::from_str::<serde_json::Value>(&raw).ok())
                .unwrap_or(serde_json::Value::Null);
            let commit = value
                .get("commit")
                .and_then(|value| value.as_str())
                .map(str::to_owned);
            match commit {
                Some(commit) if commit.len() == 40 => {
                    let present = git_commit_present(root, &commit)?;
                    let ancestor = if present {
                        git_is_ancestor(root, &commit, "HEAD")?
                    } else {
                        false
                    };
                    (Some(commit), present, ancestor)
                }
                _ => (None, false, false),
            }
        } else {
            (None, false, false)
        };

    let review_path = evidence_dir.join("review.json");
    let review_present = review_path.is_file();
    // #743: existence is not currency. `finalize` additionally requires the recorded review
    // to still match this tree (`scoped_review_is_current`), and readiness never asked — so
    // ship-status recommended `ship` in the same second `finalize` refused it, with only a
    // read-only command in between. #689 moved the verification half of this very conjunction
    // onto content and left the review half asking whether a file was on disk.
    let review_currency = change::recorded_scoped_review_currency(root, record);
    let review_status = ShipReviewStatus::resolve(review_currency.as_ref(), review_present);

    let mut blockers = Vec::new();
    let mut warnings = Vec::new();
    if matches!(
        record.state,
        ChangeState::Verifying | ChangeState::Implementing | ChangeState::Approved
    ) {
        if verification_commit.is_none() {
            blockers.push("no verification evidence recorded yet".to_string());
        } else if !verification_present {
            blockers.push(
                "verification commit is not in this repository (orphaned by squash merge, rebase, amend, or force-push); re-run change check --commit"
                    .to_string(),
            );
        } else if !change::recorded_verification_is_current(root, record) {
            // #689: the question is whether the CONTENT still matches, not whether the recorded
            // commit is still reachable. A squash-merge guarantees it is not — and squash is the
            // only strategy this repository, and most repositories, permit. Blocking on
            // reachability made a squash-merged change permanently unfinalizable while its
            // evidence was perfectly good.
            blockers.push(
                "verification evidence is stale for the current tree; re-run change check --commit before review/finalize"
                    .to_string(),
            );
        }
        if record.state == ChangeState::Verifying {
            // Three outcomes, three sentences. A DECIDED negative is a blocker, matching the
            // verification half directly above; an UNDECIDED one is a warning, because saying
            // "you are blocked" would settle #694's open question by accident. What neither
            // of them may do is stay silent and let readiness read the silence as a pass.
            match &review_currency {
                None if !review_present => warnings.push(
                    "no scoped review recorded yet; finalize requires independent review"
                        .to_string(),
                ),
                None => blockers.push(format!(
                    "the recorded scoped review cannot be read; re-run `specsync change review {} --reviewer <independent-reviewer>` before finalize",
                    record.id
                )),
                Some(change::ScopedReviewCurrency::Current) => {}
                Some(change::ScopedReviewCurrency::Stale(reason)) => blockers.push(format!(
                    "scoped review is stale — {reason}; re-run `specsync change review {} --reviewer <independent-reviewer>` before finalize",
                    record.id
                )),
                Some(change::ScopedReviewCurrency::Unavailable(reason)) => warnings.push(format!(
                    "scoped review currency could not be determined — {reason}; readiness cannot confirm it and finalize may refuse. Re-recording the review with `specsync change review {} --reviewer <independent-reviewer>` re-anchors it",
                    record.id
                )),
            }
            warnings.push(merge_before_finalize_warning(false));
        }
        if tip.tip_class == "archive_only" && record.state != ChangeState::Archived {
            warnings.push(
                "HEAD looks like an archive-only tip while this change is still active — push product/review tips first, then finalize"
                    .to_string(),
            );
        }
        if tip.tip_class == "review_only" && !review_present {
            warnings.push(
                "HEAD looks like a review-only tip but no scoped review is recorded for this change"
                    .to_string(),
            );
        }
    }

    // Ordering rules from multi-change dogfood (#487): always surface while active.
    if record.state != ChangeState::Archived {
        if !warnings
            .iter()
            .any(|warning| warning.contains("merge the PR before finalize"))
        {
            warnings.push(merge_before_finalize_warning(true));
        }
        warnings.push(
            "review then ship without an intermediate commit — committing between review and finalize stales the scoped review workspace digest"
                .to_string(),
        );
    }

    let sibling_active_ids = sibling_active_change_ids(root, &record.id);
    if !sibling_active_ids.is_empty() {
        warnings.extend(multi_active_ordering_warnings(&sibling_active_ids));
    }

    // #689: readiness is a CONTENT question, not a history one. The rest of this module settled
    // that (see `verification_is_current`); ship-status was the caller that never got the change,
    // so a squash-merged repo could never reach `ready_to_finalize`.
    // #743 finished the sentence for the review half. `review_status.current()` subsumes
    // `review_present`: only a review that was loaded AND agreed with this tree may count,
    // and an unavailable answer is not a satisfied one (#694's standard, applied to the one
    // caller that violated it).
    let verification_current = change::recorded_verification_is_current(root, record);
    let ready_to_finalize = record.state == ChangeState::Verifying
        && verification_present
        && verification_current
        && review_status.current()
        && blockers.is_empty();

    let stages = ship_stages(
        record,
        &tip,
        verification_commit.as_deref(),
        verification_present,
        verification_ancestor,
        review_status,
        ready_to_finalize,
    );

    let current_stage = stages
        .iter()
        .find(|stage| stage.get("status").and_then(|value| value.as_str()) == Some("current"))
        .cloned()
        .unwrap_or_else(|| {
            serde_json::json!({
                "id": "unknown",
                "status": "current",
                "action": lifecycle_next.clone(),
            })
        });

    let ship_next = if ready_to_finalize {
        if sibling_active_ids.is_empty() {
            format!(
                "run `specsync change ship {}` (or finalize without intermediate commits), push the archive tip, wait for CI, then merge the PR",
                record.id
            )
        } else {
            format!(
                "run `specsync change ship {}` without an intermediate commit after review; then re-check siblings ({}) before merge — never merge while any change is active",
                record.id,
                sibling_active_ids.join(", ")
            )
        }
    } else if matches!(
        record.state,
        ChangeState::Draft | ChangeState::Accepted | ChangeState::Archived
    ) {
        // The ship lane may narrow the next action; it may never contradict the
        // lifecycle state. Outside the shipping window the lane's own advice is
        // premature or spent, and obeying it fails: at draft this printed
        // `change check --commit` while the change was still in its interview, and at
        // archived it printed the same on a change that was finished (#534). Same rule
        // as the CI lane classification in #626 — narrowing is allowed, contradiction
        // is not.
        lifecycle_next.clone()
    } else {
        // A blocker says what is wrong, not what to do. This arm used to return
        // `blockers[0]` verbatim whenever any blocker existed, which put
        // "no verification evidence recorded yet" on a line whose whole job is to name
        // a runnable command. Blockers already render on their own `Blocker:` lines, so
        // repeating one here bought nothing and cost the reader their next step.
        current_stage
            .get("action")
            .and_then(|value| value.as_str())
            .unwrap_or(lifecycle_next.as_str())
            .to_string()
    };

    let trust = build_ship_trust(root, &tip);

    Ok(serde_json::json!({
        "id": record.id,
        "state": record.state,
        "tip_class": tip.tip_class,
        "tip_sha": tip.tip_sha,
        "parent_sha": tip.parent_sha,
        "trust": trust,
        "stages": stages,
        "current_stage": current_stage,
        "verification_commit": verification_commit,
        "verification_present": verification_present,
        "verification_ancestor_of_head": verification_ancestor,
        "review_present": review_present,
        "review_currency": review_status.label(),
        "review_currency_reason": review_currency.as_ref().and_then(|currency| currency.reason()),
        "ready_to_finalize": ready_to_finalize,
        "blockers": blockers,
        "warnings": warnings,
        "sibling_active_ids": sibling_active_ids,
        "lifecycle_next": lifecycle_next,
        "ship_next": ship_next,
    }))
}

/// Sibling active changes, counting the ones that could not be read.
///
/// This answers "is anything else in flight?", so it must fail closed. An
/// unreadable workspace is still an active change directory — we simply cannot
/// say which state it is in — and dropping it would report an empty field that
/// means "nothing else is in flight" when the truth is "I could not tell".
fn sibling_active_change_ids(root: &Path, id: &str) -> Vec<String> {
    let Ok(roster) = change::list_changes(root) else {
        return Vec::new();
    };
    let mut ids: Vec<String> = roster
        .records
        .into_iter()
        .filter(|record| record.id != id && record.state != ChangeState::Archived)
        .map(|record| record.id)
        .collect();
    ids.extend(
        roster
            .unreadable
            .into_iter()
            .map(|entry| entry.id)
            .filter(|entry| entry != id),
    );
    ids.sort();
    ids.dedup();
    ids
}

const SHIP_TRUST_LOCAL_GUIDANCE: &str = "After pushing a product tip, wait for trust + SpecSync implementation ready (and required product checks) to succeed before pushing a review_only or archive_only tip. Review/archive tips reuse trust from a green product parent — do not push them while product trust is still running or cancelled.";

/// Live GitHub check-run trust when online; otherwise local guidance (never blocks ship-status).
fn build_ship_trust(root: &Path, tip: &HeadTip) -> serde_json::Value {
    let local = || {
        serde_json::json!({
            "status": "local_guidance",
            "source": "local_guidance",
            "parent_sha": tip.parent_sha,
            "tip_sha": tip.tip_sha,
            "guidance": SHIP_TRUST_LOCAL_GUIDANCE,
        })
    };

    if std::env::var_os("SPECSYNC_SHIP_LOCAL_GUIDANCE").is_some() {
        return local();
    }
    let token_ok = std::env::var("GITHUB_TOKEN")
        .map(|value| !value.trim().is_empty())
        .unwrap_or(false);
    if !token_ok {
        return local();
    }
    let Some(repo) = crate::github::detect_repo(root) else {
        return local();
    };
    // Prefer parent SHA: review/archive tips reuse trust from the product parent.
    let query_sha = tip
        .parent_sha
        .as_deref()
        .or(tip.tip_sha.as_deref())
        .map(str::to_string);
    let Some(query_sha) = query_sha else {
        return serde_json::json!({
            "status": "unavailable",
            "source": "github_check_runs",
            "repo": repo,
            "parent_sha": tip.parent_sha,
            "tip_sha": tip.tip_sha,
            "guidance": SHIP_TRUST_LOCAL_GUIDANCE,
            "error": "no commit SHA available for check-run lookup",
        });
    };

    match crate::github::fetch_commit_check_summary(&repo, &query_sha) {
        Ok(summary) => {
            let checks: Vec<serde_json::Value> = summary
                .check_runs
                .iter()
                .map(|run| {
                    serde_json::json!({
                        "name": run.name,
                        "status": run.status,
                        "conclusion": run.conclusion,
                        "trust_relevant": crate::github::is_trust_relevant_check_name(&run.name),
                    })
                })
                .collect();
            let trust_relevant: Vec<serde_json::Value> = checks
                .iter()
                .filter(|check| check.get("trust_relevant").and_then(|v| v.as_bool()) == Some(true))
                .cloned()
                .collect();
            let guidance = match summary.overall.as_str() {
                "green" => format!(
                    "GitHub check-runs on {query_sha:.8} are green — safe to push a review_only or archive_only tip that reuses this parent when CI trust-reuse is enabled."
                ),
                "pending" => format!(
                    "GitHub check-runs on {query_sha:.8} are still pending — wait for trust + SpecSync implementation ready before pushing a review/archive tip."
                ),
                "failed" => format!(
                    "GitHub check-runs on {query_sha:.8} failed or cancelled — do not push a review/archive tip until the product parent is green again."
                ),
                "empty" => format!(
                    "No check-runs returned for {query_sha:.8} yet — wait for CI to start, or confirm the SHA was pushed to GitHub."
                ),
                other => format!(
                    "GitHub check-runs on {query_sha:.8} reported status `{other}`. {SHIP_TRUST_LOCAL_GUIDANCE}"
                ),
            };
            serde_json::json!({
                "status": summary.overall,
                "source": "github_check_runs",
                "repo": summary.repo,
                "sha": summary.sha,
                "parent_sha": tip.parent_sha,
                "tip_sha": tip.tip_sha,
                "checks": checks,
                "trust_relevant_checks": trust_relevant,
                "guidance": guidance,
            })
        }
        Err(error) => serde_json::json!({
            "status": "unavailable",
            "source": "github_check_runs",
            "repo": repo,
            "sha": query_sha,
            "parent_sha": tip.parent_sha,
            "tip_sha": tip.tip_sha,
            "error": error,
            "guidance": SHIP_TRUST_LOCAL_GUIDANCE,
        }),
    }
}

/// Point a new change at what its modules already learned.
///
/// The other end of the loop `finalize` closes. Lessons are folded into `specs/<module>/context.md`
/// at archival precisely so the NEXT change to that module can read them — but nothing surfaced
/// them, so they accumulated where nobody looked.
///
/// Deliberately a pointer and not a dump: the file can be long, and a wall of text at creation
/// time gets scrolled past. Naming it with its size is enough to make reading it a choice the
/// author knows they are making.
fn print_accumulated_lessons(root: &Path, record: &change::ChangeRecord) {
    let found = change::accumulated_lessons(root, &record.affected_specs);
    if found.is_empty() {
        return;
    }
    println!(
        "\n  {} what these modules already learned:",
        "Lessons:".bold()
    );
    for (path, lines) in found {
        println!("    {path} ({lines} line(s)) — read before scoping this change");
    }
}

/// What to do after `ship` finalizes.
///
/// `finalize` named the lesson fold-back and `ship` did not, so on the verb the tool actually
/// recommends the bundle was written and nothing said it existed — knowledge produced where
/// nobody looks, which is the exact failure the lessons loop was built to end. It reappeared
/// inside the loop because the two verbs each built their own next-action string.
///
/// Pure so the wiring is pinned by a test rather than by having run it once: the regression this
/// guards is a future edit to one verb's guidance that forgets the other.
///
/// Fold-back comes FIRST because it is the step a merge makes irreversible — after the merge the
/// change is inert history and the material is archived where nobody reads it.
fn ship_next_action(
    push: bool,
    wait: bool,
    siblings_before: &[String],
    fold_targets: &[String],
    bundle: &str,
) -> String {
    let remaining = if push && wait {
        if siblings_before.is_empty() {
            "merge the PR on GitHub when Required CI is green".to_string()
        } else {
            format!(
                "re-run `change check --commit` on remaining active changes ({}) before merge",
                siblings_before.join(", ")
            )
        }
    } else if push {
        if siblings_before.is_empty() {
            "wait for CI, then merge the PR (or re-run `change ship --wait`)".to_string()
        } else {
            format!(
                "wait for CI; then re-run `change check --commit` on remaining active changes ({})",
                siblings_before.join(", ")
            )
        }
    } else if siblings_before.is_empty() {
        "commit if needed, push the archive tip, wait for CI, then merge the PR".to_string()
    } else {
        format!(
            "commit if needed, push the archive tip, wait for CI; then re-run `change check --commit` on remaining active changes ({}) — do not merge while any change is active",
            siblings_before.join(", ")
        )
    };
    // The archive still writes the lesson bundle. Naming the fold in next_action
    // made a documentation convention a merge gate. Keep the remaining guidance.
    let _ = (fold_targets, bundle);
    remaining
}

/// Name the fold-back step archival exists for, then the merge.
///
/// Archival is the only point at which the system compounds rather than merely records: knowledge
/// moves out of the change, which is about to become inert history, and into the spec, which is
/// read by everyone who touches the module next. A change's own `context.md` is archived and read
/// by nobody; `specs/<module>/context.md` is read before every future change to that module.
///
/// SpecSync does not write the lessons and must not — it would have to shell out to a particular
/// agent. It does not need to: whoever just ran `finalize` is right there. So name the step and
/// point at the material. `next_action` is the mechanism the lifecycle already uses everywhere,
/// and drill 032 confirms agents follow it to termination.
fn lessons_next_action(root: &Path, id: &str, archive: &Path) -> String {
    let _ = (change::lesson_fold_targets(root, id), archive);
    "merge the PR on GitHub".to_string()
}

/// What merging before `finalize` actually costs.
///
/// The warning used to say it "orphans verification evidence and strands the change", which
/// prices the loss as ONE record — the reader's own, recoverable. Measured on a real repository,
/// the cost is larger and lands on other people's work: an unfinalized change never reaches
/// `accepted` or `archived`, so it never becomes an "accepted or archived successor", and every
/// EARLIER accepted change sharing a delivery input with it can no longer archive.
///
/// That second-order effect is why an accepted pile can grow without any single merge decision
/// looking wrong. Each one is individually small and locally recoverable; the aggregate is a
/// lifecycle that cannot drain. Say the real price at the moment the decision is made.
fn merge_before_finalize_warning(still_active: bool) -> String {
    let opening = if still_active {
        "do not merge the PR while this change is still active"
    } else {
        "do not merge the PR before finalize"
    };
    format!(
        "{opening} — merging first orphans its verification evidence AND blocks every earlier \
accepted change sharing a delivery input from archiving, until this one is finalized or those \
are reopened"
    )
}

/// Warnings when more than one change is active on the same branch/PR.
fn multi_active_ordering_warnings(sibling_ids: &[String]) -> Vec<String> {
    if sibling_ids.is_empty() {
        return Vec::new();
    }
    vec![
        format!(
            "other active changes remain ({}): finalize one at a time — archiving this change immediately stales sibling verification",
            sibling_ids.join(", ")
        ),
        "do not batch reviews across changes — each review binds a workspace digest; recording another review first stales the prior one".to_string(),
        "do not merge the PR while any SDD change is still active — merge only after every change on the PR is archived".to_string(),
    ]
}

/// What ship-status knows about the recorded scoped review.
///
/// `ship_stages` and `ready_to_finalize` both used to take a bare `review_present: bool`,
/// which is the existence-only question #743 is about: `finalize` requires the review to
/// still be CURRENT, so a stage reading "done" and a readiness flag reading `true` were both
/// describing a review the very next command would reject. Presence and currency are kept as
/// separate variants here because they are separate questions — a missing review and a stale
/// one need different sentences, and an *unavailable* answer needs a third (#694).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ShipReviewStatus {
    /// No `review.json` on disk.
    Missing,
    /// A `review.json` exists but no usable review record could be loaded from it. `finalize`
    /// fails on the same read, so reporting it as recorded-and-fine would disagree with it.
    Unreadable,
    /// Recorded, and every currency check agreed with this tree.
    Current,
    /// Recorded, and decidably out of date — a digest moved, or the descendant walk caught a
    /// forbidden change.
    Stale,
    /// Recorded, and its currency could not be determined at all. This is #694's case, and
    /// this variant exists so readiness can decline to answer instead of answering `true`.
    Unavailable,
}

impl ShipReviewStatus {
    fn resolve(currency: Option<&change::ScopedReviewCurrency>, present: bool) -> Self {
        match currency {
            Some(change::ScopedReviewCurrency::Current) => Self::Current,
            Some(change::ScopedReviewCurrency::Stale(_)) => Self::Stale,
            Some(change::ScopedReviewCurrency::Unavailable(_)) => Self::Unavailable,
            None if present => Self::Unreadable,
            None => Self::Missing,
        }
    }

    /// Whether a review artifact exists at all. Deliberately still true for `Unreadable`:
    /// the JSON `review_present` field has always answered "is there a file", and an archived
    /// package whose attempt ledger did not travel is still a change that was reviewed.
    fn present(self) -> bool {
        !matches!(self, Self::Missing)
    }

    /// The only variant readiness may treat as satisfied.
    fn current(self) -> bool {
        matches!(self, Self::Current)
    }

    fn label(self) -> &'static str {
        match self {
            Self::Missing => "missing",
            Self::Unreadable => "unreadable",
            Self::Current => "current",
            Self::Stale => "stale",
            Self::Unavailable => "unavailable",
        }
    }
}

fn ship_stages(
    record: &ChangeRecord,
    tip: &HeadTip,
    verification_commit: Option<&str>,
    verification_present: bool,
    verification_ancestor: bool,
    review: ShipReviewStatus,
    ready_to_finalize: bool,
) -> Vec<serde_json::Value> {
    let id = record.id.as_str();
    let verified = verification_present && verification_ancestor && verification_commit.is_some();
    let mut stages = Vec::new();

    let product_done = verified
        && matches!(
            record.state,
            ChangeState::Verifying | ChangeState::Accepted | ChangeState::Archived
        );
    stages.push(serde_json::json!({
        "id": "product_tip",
        "title": "Product tip (full CI)",
        "status": if product_done { "done" } else { "current" },
        "sha": verification_commit,
        "action": if product_done {
            "product verification evidence is on an ancestor of HEAD".to_string()
        } else {
            format!("run `specsync change check {id} --commit`, push the product tip, wait for trust + implementation ready")
        },
    }));

    // The stage tracks currency, not existence (#743). A `done` review tip beside a
    // `ready_to_finalize: false` is the same recommend-then-refuse contradiction one line
    // lower, and it is also what leaves `current_stage` empty — which drops `ship_next` back
    // to the generic lifecycle line instead of naming the recovery.
    let review_status = if review.current() {
        "done"
    } else if product_done {
        "current"
    } else {
        "pending"
    };
    stages.push(serde_json::json!({
        "id": "review_tip",
        "title": "Review tip (trust reuse from product parent)",
        "status": review_status,
        "action": if review.current() {
            "scoped review is recorded and current".to_string()
        } else if product_done && review.present() {
            format!(
                "re-record the independent review: `specsync change review {id} --reviewer <other>` — the recorded review is `{}` against this tree, and finalize requires a current one",
                review.label()
            )
        } else if product_done {
            format!(
                "record independent review: `specsync change review {id} --reviewer <other>` then push a review_only tip if CI requires it; wait for trust reuse"
            )
        } else {
            "complete product tip first".to_string()
        },
    }));

    let archive_status = if record.state == ChangeState::Archived {
        "done"
    } else if ready_to_finalize {
        "current"
    } else {
        "pending"
    };
    stages.push(serde_json::json!({
        "id": "archive_tip",
        "title": "Archive tip (finalize on the same PR)",
        "status": archive_status,
        "action": if record.state == ChangeState::Archived {
            "change is archived; merge the PR".to_string()
        } else if ready_to_finalize {
            format!(
                "run `specsync change ship {id}` (or finalize), push the archive tip, wait for CI, then merge"
            )
        } else {
            "complete product tip and independent review first".to_string()
        },
    }));

    stages.push(serde_json::json!({
        "id": "merge",
        "title": "Merge the PR",
        "status": if record.state == ChangeState::Archived { "current" } else { "pending" },
        "action": "merge only after finalize — merging an unfinalized change orphans verification evidence",
        "head_tip_class": tip.tip_class,
    }));

    stages
}

#[derive(Debug, Clone)]
struct HeadTip {
    tip_class: String,
    tip_sha: Option<String>,
    parent_sha: Option<String>,
}

/// Classify HEAD the way CI tip lanes roughly do: archive_only, review_only, product, or other.
fn classify_head_tip(root: &Path) -> Result<HeadTip, String> {
    let tip_sha = git_rev_parse(root, "HEAD").ok();
    let parent_sha = tip_sha
        .as_ref()
        .and_then(|_| git_rev_parse(root, "HEAD^").ok());

    let paths = if let Some(tip) = tip_sha.as_ref() {
        if let Some(parent) = parent_sha.as_ref() {
            git_diff_name_only(root, parent, tip).unwrap_or_default()
        } else {
            git_show_name_only(root, tip).unwrap_or_default()
        }
    } else {
        Vec::new()
    };

    // Dirty working tree paths also count as product when present.
    let mut dirty = git_status_name_only(root).unwrap_or_default();
    let mut all_paths = paths;
    all_paths.append(&mut dirty);
    all_paths.sort();
    all_paths.dedup();

    let tip_class = classify_paths(&all_paths).to_string();
    Ok(HeadTip {
        tip_class,
        tip_sha,
        parent_sha,
    })
}

fn classify_paths(paths: &[String]) -> &'static str {
    if paths.is_empty() {
        return "other";
    }
    let mut has_archive = false;
    let mut has_review = false;
    let mut has_product = false;
    for path in paths {
        if path.starts_with(".specsync/archive/changes/") {
            has_archive = true;
            continue;
        }
        if path.starts_with(".specsync/changes/")
            && (path.ends_with("/review.json")
                || path.ends_with("/review-attempts.json")
                || path.ends_with("review.json")
                || path.ends_with("review-attempts.json"))
        {
            has_review = true;
            continue;
        }
        // Lifecycle metadata alone is not a full product tip.
        if path == ".specsync/change-sequence.json"
            || path.starts_with(".specsync/changes/")
                && (path.ends_with("/verification.json")
                    || path.ends_with("/verification-attempts.json")
                    || path.ends_with("/state.json")
                    || path.ends_with("/approvals.json"))
        {
            continue;
        }
        has_product = true;
    }
    if has_product {
        "product"
    } else if has_archive && !has_review {
        "archive_only"
    } else if has_review && !has_archive {
        "review_only"
    } else if has_archive && has_review {
        "product"
    } else {
        "other"
    }
}

fn print_ship_status(
    root: &Path,
    record: &ChangeRecord,
    format: OutputFormat,
) -> Result<(), String> {
    let report = ship_status_report(root, record)?;
    match format {
        OutputFormat::Json => {
            print_json(&report);
            Ok(())
        }
        _ => {
            println!(
                "{}  {}  tip={}  ({})",
                report["id"].as_str().unwrap_or("?"),
                report["state"].as_str().unwrap_or("?"),
                report["tip_class"].as_str().unwrap_or("?"),
                if report["ready_to_finalize"].as_bool() == Some(true) {
                    "ready to finalize".green().to_string()
                } else {
                    "not ready".yellow().to_string()
                }
            );
            if let Some(sha) = report["tip_sha"].as_str() {
                println!("  HEAD: {}", &sha[..8.min(sha.len())]);
            }
            if let Some(parent) = report["parent_sha"].as_str() {
                println!("  Parent: {}", &parent[..8.min(parent.len())]);
            }
            if let Some(commit) = report["verification_commit"].as_str() {
                let tip = if report["verification_present"].as_bool() != Some(true) {
                    "absent from repository"
                } else if report["verification_ancestor_of_head"].as_bool() != Some(true) {
                    "not ancestor of HEAD"
                } else {
                    "ancestor of HEAD"
                };
                println!(
                    "  Verification: {} ({})",
                    &commit[..8.min(commit.len())],
                    tip
                );
            } else {
                println!("  Verification: none");
            }
            // Existence alone was the defect (#743); the line that reports the review has
            // to report which of the three answers it got, or a reader sees "recorded" for a
            // review `finalize` is about to reject.
            println!(
                "  Review: {}",
                match report["review_currency"].as_str().unwrap_or("missing") {
                    "missing" => "missing".to_string(),
                    currency => format!("recorded ({currency})"),
                }
            );
            {
                let status = report
                    .pointer("/trust/status")
                    .and_then(|value| value.as_str())
                    .unwrap_or("?");
                let source = report
                    .pointer("/trust/source")
                    .and_then(|value| value.as_str())
                    .unwrap_or("?");
                let sha = report
                    .pointer("/trust/sha")
                    .and_then(|value| value.as_str())
                    .map(|s| format!(" @ {}", &s[..8.min(s.len())]))
                    .unwrap_or_default();
                println!("  Trust: {status} ({source}{sha})");
                if let Some(guidance) = report
                    .pointer("/trust/guidance")
                    .and_then(|value| value.as_str())
                {
                    println!("    {guidance}");
                }
                if let Some(error) = report
                    .pointer("/trust/error")
                    .and_then(|value| value.as_str())
                {
                    println!("    lookup: {error}");
                }
            }
            if let Some(stages) = report["stages"].as_array() {
                println!("  Stages:");
                for stage in stages {
                    let id = stage
                        .get("id")
                        .and_then(|value| value.as_str())
                        .unwrap_or("?");
                    let status = stage
                        .get("status")
                        .and_then(|value| value.as_str())
                        .unwrap_or("?");
                    let action = stage
                        .get("action")
                        .and_then(|value| value.as_str())
                        .unwrap_or("");
                    println!("    - [{status}] {id}: {action}");
                }
            }
            if let Some(blockers) = report["blockers"].as_array() {
                for blocker in blockers {
                    if let Some(text) = blocker.as_str() {
                        println!("  Blocker: {}", text.red());
                    }
                }
            }
            if let Some(warnings) = report["warnings"].as_array() {
                for warning in warnings {
                    if let Some(text) = warning.as_str() {
                        println!("  Warning: {}", text.yellow());
                    }
                }
            }
            if let Some(next) = report["ship_next"].as_str() {
                println!("  Next: {next}");
            }
            Ok(())
        }
    }
}

fn run_ship(
    root: &Path,
    id: Option<&str>,
    dry_run: bool,
    push: bool,
    wait: bool,
    wait_timeout_secs: u64,
    format: OutputFormat,
) -> Result<(), String> {
    if wait && !push && !dry_run {
        // Waiting without push still polls HEAD (already-pushed archive tips).
    }
    if dry_run && (push || wait) {
        return Err("ship --dry-run cannot be combined with --push or --wait".into());
    }

    let record = match id {
        Some(id) => change::load_change(root, id)?,
        None => {
            let roster = change::list_changes(root)?;
            // Inferring "the one active change" from a partial roster can ship
            // the wrong change, or ship while another is still in flight. With
            // any workspace unreadable there is no safe inference to make.
            if let Some(unreadable) = roster.unreadable.first() {
                return Err(format!(
                    "cannot infer which change to ship while a workspace is unreadable: {}; pass an explicit change id",
                    unreadable.reason
                ));
            }
            match roster.records.as_slice() {
                [] => return Err("no active change to ship; pass an explicit change id".into()),
                [single] => single.clone(),
                _ => {
                    return Err(
                        "multiple active changes; pass an explicit id: `specsync change ship <ID>`"
                            .into(),
                    );
                }
            }
        }
    };

    if record.state == ChangeState::Archived {
        let report = ship_status_report(root, &record)?;
        if push || wait {
            let push_result = if push {
                Some(ship_commit_and_push_archive(root, &record.id)?)
            } else {
                None
            };
            let wait_result = if wait {
                Some(wait_for_head_check_runs(root, wait_timeout_secs, format)?)
            } else {
                None
            };
            match format {
                OutputFormat::Json => print_json(&serde_json::json!({
                    "id": record.id,
                    "status": "already_archived",
                    "report": report,
                    "push": push_result,
                    "wait": wait_result,
                })),
                _ => {
                    println!("{} {} is already archived", "✓".green(), record.id);
                    if let Some(push_result) = push_result.as_ref() {
                        println!("  Push: {push_result}");
                    }
                    if let Some(wait_result) = wait_result.as_ref() {
                        println!(
                            "  Wait: {}",
                            wait_result
                                .get("status")
                                .and_then(|v| v.as_str())
                                .unwrap_or("?")
                        );
                    }
                    if !push && !wait {
                        println!("  Next: merge the PR on GitHub");
                    }
                }
            }
            return Ok(());
        }
        match format {
            OutputFormat::Json => print_json(&serde_json::json!({
                "id": record.id,
                "status": "already_archived",
                "report": report,
            })),
            _ => {
                println!(
                    "{} {} is already archived — merge the PR on GitHub",
                    "✓".green(),
                    record.id
                );
            }
        }
        return Ok(());
    }

    let report = ship_status_report(root, &record)?;
    let ready = report["ready_to_finalize"].as_bool() == Some(true);
    let tip_class = report["tip_class"].as_str().unwrap_or("other");
    let trust_status = report
        .pointer("/trust/status")
        .and_then(|value| value.as_str())
        .unwrap_or("local_guidance");

    // Soft preflight: live trust is advisory unless --wait forces a green parent later.
    if matches!(trust_status, "pending" | "failed" | "empty") {
        match format {
            OutputFormat::Json => {}
            _ => {
                println!(
                    "{} parent/product tip trust is `{trust_status}` — prefer waiting for green CI before archive tip push (see ship-status trust)",
                    "⚠".yellow()
                );
            }
        }
    }

    if !ready {
        match format {
            OutputFormat::Json => print_json(&serde_json::json!({
                "id": record.id,
                "status": "blocked",
                "tip_class": tip_class,
                "report": report,
            })),
            _ => {
                let _ = print_ship_status(root, &record, format);
                println!(
                    "{} ship blocked — resolve blockers above, then re-run `specsync change ship {}`",
                    "✗".red(),
                    record.id
                );
            }
        }
        return Err(format!("change {} is not ready to ship", record.id));
    }

    if dry_run {
        match format {
            OutputFormat::Json => print_json(&serde_json::json!({
                "id": record.id,
                "status": "ready",
                "dry_run": true,
                "tip_class": tip_class,
                "report": report,
                "next": format!("run `specsync change ship {}` without --dry-run to finalize", record.id),
            })),
            _ => {
                let _ = print_ship_status(root, &record, format);
                println!(
                    "{} ready to finalize (dry-run) — re-run without --dry-run to ship",
                    "✓".green()
                );
            }
        }
        return Ok(());
    }

    let siblings_before = report
        .get("sibling_active_ids")
        .and_then(|value| value.as_array())
        .map(|ids| {
            ids.iter()
                .filter_map(|value| value.as_str().map(str::to_owned))
                .collect::<Vec<_>>()
        })
        .unwrap_or_default();

    // Finalize mutates the workspace; drop the read scope by ending this block first.
    let path = change::finalize_change(root, &record.id)?;

    let mut push_result = None;
    let mut wait_result = None;
    if push {
        push_result = Some(ship_commit_and_push_archive(root, &record.id)?);
    }
    if wait {
        wait_result = Some(wait_for_head_check_runs(root, wait_timeout_secs, format)?);
    }

    let next = ship_next_action(
        push,
        wait,
        &siblings_before,
        &change::lesson_fold_targets(root, &record.id),
        &path.join(change::LESSON_BUNDLE_FILE).display().to_string(),
    );
    match format {
        OutputFormat::Json => print_json(&serde_json::json!({
            "id": record.id,
            "status": "finalized",
            "archived": path,
            "tip_class": "archive_only",
            "lesson_bundle": path.join(change::LESSON_BUNDLE_FILE),
            "sibling_active_ids": siblings_before,
            "push": push_result,
            "wait": wait_result,
            "next": next,
        })),
        _ => {
            println!("{} {} finalized on this PR", "✓".green(), record.id);
            println!("  Archive: {}", path.display());
            if let Some(push_result) = push_result.as_ref() {
                println!("  Push: {push_result}");
            }
            if let Some(wait_result) = wait_result.as_ref() {
                println!(
                    "  Wait: {}",
                    wait_result
                        .get("status")
                        .and_then(|v| v.as_str())
                        .unwrap_or("?")
                );
            }
            println!("  Next: {next}");
            if !siblings_before.is_empty() {
                println!(
                    "  {}",
                    "Warning: sibling active changes still need their own check → review → ship cycle (archive tips stale them)."
                        .yellow()
                );
            }
        }
    }
    Ok(())
}

/// Commit archive package (if dirty) and push the current branch.
fn ship_commit_and_push_archive(root: &Path, id: &str) -> Result<String, String> {
    git_commit_all(root, &format!("chore(lifecycle): archive {id}"))?;
    run_git(root, &["push"])?;
    let sha = git_rev_parse(root, "HEAD").unwrap_or_else(|_| "HEAD".into());
    Ok(format!("pushed archive tip {sha:.8}"))
}

/// Poll GitHub check-runs for HEAD until green, failed, empty, timeout, or offline.
fn wait_for_head_check_runs(
    root: &Path,
    timeout_secs: u64,
    format: OutputFormat,
) -> Result<serde_json::Value, String> {
    let quiet = matches!(format, OutputFormat::Json);
    let say = |message: &str| {
        if !quiet {
            println!("{message}");
        }
    };

    if std::env::var_os("SPECSYNC_SHIP_LOCAL_GUIDANCE").is_some() {
        return Ok(serde_json::json!({
            "status": "local_guidance",
            "detail": "SPECSYNC_SHIP_LOCAL_GUIDANCE set; skipped wait",
        }));
    }
    let token_ok = std::env::var("GITHUB_TOKEN")
        .map(|value| !value.trim().is_empty())
        .unwrap_or(false);
    if !token_ok {
        return Ok(serde_json::json!({
            "status": "local_guidance",
            "detail": "GITHUB_TOKEN unset; cannot wait on check-runs",
        }));
    }
    let Some(repo) = crate::github::detect_repo(root) else {
        return Ok(serde_json::json!({
            "status": "unavailable",
            "detail": "no GitHub remote for check-run wait",
        }));
    };
    let sha = git_rev_parse(root, "HEAD")?;
    let deadline = std::time::Instant::now() + std::time::Duration::from_secs(timeout_secs.max(1));
    let mut last_overall = String::from("pending");
    let mut polls = 0u32;
    say(&format!(
        "Waiting up to {timeout_secs}s for check-runs on {repo}@{sha:.8}…"
    ));
    while std::time::Instant::now() < deadline {
        polls += 1;
        let overall = match crate::github::fetch_commit_check_summary(&repo, &sha) {
            Ok(summary) => {
                say(&format!("  poll {polls}: {}", summary.overall));
                match summary.overall.as_str() {
                    "green" => {
                        return Ok(serde_json::json!({
                            "status": "green",
                            "repo": repo,
                            "sha": sha,
                            "polls": polls,
                            "checks": summary.check_runs.len(),
                        }));
                    }
                    "failed" => {
                        return Err(format!(
                            "check-runs failed on {repo}@{sha:.8} after {polls} poll(s)"
                        ));
                    }
                    "empty" | "pending" => summary.overall,
                    other => {
                        return Ok(serde_json::json!({
                            "status": other,
                            "repo": repo,
                            "sha": sha,
                            "polls": polls,
                        }));
                    }
                }
            }
            Err(error) => {
                say(&format!("  poll {polls}: unavailable ({error})"));
                "unavailable".to_string()
            }
        };
        last_overall = overall;
        std::thread::sleep(std::time::Duration::from_secs(10));
    }
    Err(format!(
        "timed out after {timeout_secs}s waiting for check-runs on {repo}@{sha:.8} (last={last_overall})"
    ))
}

fn git_commit_present(root: &Path, commit: &str) -> Result<bool, String> {
    let output = std::process::Command::new("git")
        .args(["cat-file", "-e", &format!("{commit}^{{commit}}")])
        .current_dir(root)
        .output()
        .map_err(|error| format!("git cat-file failed: {error}"))?;
    Ok(output.status.success())
}

fn git_is_ancestor(root: &Path, maybe_ancestor: &str, tip: &str) -> Result<bool, String> {
    let output = std::process::Command::new("git")
        .args(["merge-base", "--is-ancestor", maybe_ancestor, tip])
        .current_dir(root)
        .output()
        .map_err(|error| format!("git merge-base failed: {error}"))?;
    Ok(output.status.success())
}

fn git_rev_parse(root: &Path, rev: &str) -> Result<String, String> {
    let output = std::process::Command::new("git")
        .args(["rev-parse", "--verify", rev])
        .current_dir(root)
        .output()
        .map_err(|error| format!("git rev-parse failed: {error}"))?;
    if !output.status.success() {
        return Err(format!("git rev-parse {rev} failed"));
    }
    Ok(String::from_utf8_lossy(&output.stdout).trim().to_string())
}

fn git_diff_name_only(root: &Path, from: &str, to: &str) -> Result<Vec<String>, String> {
    let output = std::process::Command::new("git")
        .args(["diff", "--name-only", from, to])
        .current_dir(root)
        .output()
        .map_err(|error| format!("git diff failed: {error}"))?;
    if !output.status.success() {
        return Err("git diff --name-only failed".into());
    }
    Ok(String::from_utf8_lossy(&output.stdout)
        .lines()
        .filter(|line| !line.is_empty())
        .map(str::to_owned)
        .collect())
}

fn git_show_name_only(root: &Path, commit: &str) -> Result<Vec<String>, String> {
    let output = std::process::Command::new("git")
        .args(["show", "--pretty=format:", "--name-only", commit])
        .current_dir(root)
        .output()
        .map_err(|error| format!("git show failed: {error}"))?;
    if !output.status.success() {
        return Err("git show --name-only failed".into());
    }
    Ok(String::from_utf8_lossy(&output.stdout)
        .lines()
        .filter(|line| !line.is_empty())
        .map(str::to_owned)
        .collect())
}

fn git_status_name_only(root: &Path) -> Result<Vec<String>, String> {
    let output = std::process::Command::new("git")
        .args(["status", "--porcelain", "-z"])
        .current_dir(root)
        .output()
        .map_err(|error| format!("git status failed: {error}"))?;
    if !output.status.success() {
        return Err("git status failed".into());
    }
    // Porcelain -z: XY path\0, renames have extra path.
    let mut paths = Vec::new();
    for entry in output.stdout.split(|byte| *byte == 0) {
        if entry.len() < 4 {
            continue;
        }
        let text = String::from_utf8_lossy(entry);
        // "XY path" or "R  old\0new" handled loosely: take path after first three bytes.
        let path = text.get(3..).unwrap_or("").trim();
        if !path.is_empty() {
            paths.push(path.to_string());
        }
    }
    Ok(paths)
}

fn unreadable_json(roster: &change::ChangeRoster) -> Vec<serde_json::Value> {
    roster
        .unreadable
        .iter()
        .map(|entry| serde_json::json!({ "id": entry.id, "error": entry.reason }))
        .collect()
}

fn print_unreadable_rows(roster: &change::ChangeRoster) {
    for entry in &roster.unreadable {
        println!("{}  unreadable  {}", entry.id, entry.reason);
    }
}

/// The exit status for a partial roster.
///
/// A degraded roster must not exit 0. `change audit` already exits 1 on the very
/// same tree, and two commands disagreeing about whether one repository is
/// healthy is the defect this codebase keeps re-learning (#576). The rows are
/// printed first, so the operator sees every healthy change *and* the failure.
fn unreadable_error(roster: &change::ChangeRoster) -> Result<(), String> {
    match roster.unreadable.len() {
        0 => Ok(()),
        1 => Err(roster.unreadable[0].reason.clone()),
        count => Err(format!(
            "{count} active change workspaces could not be read; the listing above is incomplete"
        )),
    }
}

/// Print the roster, then fail if any workspace could not be read.
///
/// Splitting this from [`print_records`] keeps one rule in one place: healthy
/// rows are always printed, the unreadable ones are always named, and the exit
/// status always reflects that the view is partial.
fn print_roster(
    root: &Path,
    roster: &change::ChangeRoster,
    format: OutputFormat,
    strict: bool,
) -> Result<(), String> {
    if matches!(format, OutputFormat::Json) && roster.is_degraded() {
        // A bare array cannot say "and there were three I could not read", so a
        // degraded roster is reported as an object. Healthy rosters keep the
        // historical array shape exactly, which is every project that is not
        // already being lied to.
        let summaries: Vec<_> = roster
            .records
            .iter()
            .map(|record| change::summarize_change_with_strict(root, record, strict))
            .collect();
        let error = unreadable_error(roster).err();
        print_json(&serde_json::json!({
            "changes": summaries,
            "unreadable": unreadable_json(roster),
            "error": error,
        }));
        // Exit here rather than returning Err. `cmd_change`'s tail handler prints
        // its own `{"error": …}` document in JSON mode, and a second document
        // concatenated after this one makes stdout unparseable — the failure this
        // whole change exists to stop, reintroduced one layer up.
        process::exit(1);
    }
    if !roster.records.is_empty() || !roster.is_degraded() {
        print_records(root, &roster.records, format, strict)?;
    }
    print_unreadable_rows(roster);
    unreadable_error(roster)
}

fn print_records(
    root: &Path,
    records: &[ChangeRecord],
    format: OutputFormat,
    strict: bool,
) -> Result<(), String> {
    match format {
        OutputFormat::Json => {
            let summaries: Vec<_> = records
                .iter()
                .map(|record| change::summarize_change_with_strict(root, record, strict))
                .collect();
            print_json(&summaries);
            Ok(())
        }
        _ if records.is_empty() => {
            println!("No active SDD changes.");
            Ok(())
        }
        _ => {
            // Text list view avoids digest-bearing summarize loaders (cleartext-logging).
            // Preflight every record before printing any successful lifecycle projection.
            // Otherwise a later invalid ledger would leave misleading earlier success rows.
            for record in records {
                ensure_text_correction_ledger_valid(root, record)?;
            }
            for record in records {
                let questions = change::next_questions(record);
                let id = record.id.clone();
                let title = record.title.clone();
                let state = record.state.as_str().to_owned();
                let next = text_mode_next_action(root, record, &questions);
                println!("{:<14}  {state:<13}  {title}  next: {next}", id.bold());
            }
            Ok(())
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

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    /// The floor must be WIRED, not merely present (#533).
    ///
    /// `floor_sequence_ledger_to_committed` has its own unit tests, but those
    /// exercise the function directly. Nothing asserted that `git_commit_all`
    /// actually calls it, so deleting the call left the entire suite green
    /// while every lifecycle commit went back to staging a stale ledger over a
    /// higher committed mark — the exact regression #533 is about.
    ///
    /// This test drives the real staging path and inspects what landed in the
    /// commit, so it fails if the call is removed.
    #[test]
    fn git_commit_all_raises_a_stale_ledger_before_staging_it() {
        use std::process::Command;
        let temp = TempDir::new().expect("temp project");
        let root = temp.path();
        let git = |args: &[&str]| {
            assert!(
                Command::new("git")
                    .args(args)
                    .current_dir(root)
                    .status()
                    .expect("git")
                    .success(),
                "git {args:?}"
            );
        };
        let write_ledger = |sequence: u64| {
            std::fs::create_dir_all(root.join(".specsync")).unwrap();
            std::fs::write(
                root.join(".specsync/change-sequence.json"),
                format!(
                    "{{\n  \"schema_version\": 1,\n  \"sequence\": {sequence},\n  \"id\": \"CHG-{sequence:04}-fixture\",\n  \"acknowledged_collisions\": []\n}}\n"
                ),
            )
            .unwrap();
        };
        let committed_sequence = || -> u64 {
            let out = Command::new("git")
                .args(["show", "HEAD:.specsync/change-sequence.json"])
                .current_dir(root)
                .output()
                .expect("git show");
            let text = String::from_utf8_lossy(&out.stdout);
            let value: serde_json::Value = serde_json::from_str(&text).expect("ledger json");
            value["sequence"].as_u64().expect("sequence")
        };

        git(&["init", "-b", "main"]);
        git(&["config", "user.email", "test@example.com"]);
        git(&["config", "user.name", "Test"]);
        write_ledger(3);
        std::fs::write(root.join("README.md"), "base\n").unwrap();
        git(&["add", "."]);
        git(&["commit", "-m", "committed high-water mark"]);
        assert_eq!(committed_sequence(), 3);

        // The precondition: a ledger written before the branch caught up.
        write_ledger(1);
        std::fs::write(root.join("README.md"), "work\n").unwrap();

        git_commit_all(root, "lifecycle commit").expect("commit");

        assert_eq!(
            committed_sequence(),
            3,
            "the staging path must raise the stale ledger before `git add -A`; \
committing 1 over a committed 3 is the #533 regression, and it is what happens \
if the floor call is removed from this function"
        );
    }

    #[test]
    fn draft_text_surfaces_require_complete_artifacts_before_approval() {
        let temp = TempDir::new().expect("temp project");
        let root = temp.path();
        let mut record = change::create_change(
            root,
            CreateChangeRequest {
                description: "Clarify contributor guidance".into(),
                kind: ChangeKind::Documentation,
                affected_specs: Vec::new(),
                affected_paths: vec!["src/commands/change.rs".into()],
                requested_artifacts: Vec::new(),
                no_spec_change: true,
                rationale: Some(
                    "Documentation-only behavior does not alter a technical contract".into(),
                ),
            },
        )
        .expect("create draft");
        for (question, answer) in [
            (
                "acceptance_criteria",
                "Every draft text surface requires complete artifacts before approval",
            ),
            ("public_contract", "no"),
            ("architecture_risk", "no"),
        ] {
            record = change::answer_question(root, &record.id, question, answer)
                .expect("answer interview question");
        }
        let questions = change::next_questions(&record);
        assert!(questions.is_empty(), "interview must be complete");

        for surface in ["status", "show", "list"] {
            let next = text_mode_next_action(root, &record, &questions);
            assert!(
                next.contains("complete selected artifacts"),
                "{surface} text recommended the wrong next action: {next}"
            );
            assert!(
                !next.contains("change approve"),
                "{surface} text recommended premature approval: {next}"
            );
        }
    }

    fn draft_fixture(root: &Path) -> ChangeRecord {
        change::write_default_policy(root, Vec::new()).expect("write default policy");
        change::create_change(
            root,
            CreateChangeRequest {
                description: "Ship status next action".into(),
                kind: ChangeKind::Documentation,
                affected_specs: Vec::new(),
                affected_paths: vec!["README.md".into()],
                requested_artifacts: Vec::new(),
                no_spec_change: true,
                rationale: Some("Command test fixture".into()),
            },
        )
        .expect("create draft")
    }

    fn git_in(root: &Path, args: &[&str]) {
        assert!(
            std::process::Command::new("git")
                .args(args)
                .current_dir(root)
                .status()
                .unwrap()
                .success(),
            "git {args:?}"
        );
    }

    /// A git project with a committed base tree, ready for a change to be driven through it.
    fn git_project_fixture(root: &Path) {
        git_in(root, &["init", "-b", "main"]);
        git_in(root, &["config", "user.email", "t@example.com"]);
        git_in(root, &["config", "user.name", "T"]);
        fs::write(root.join("README.md"), "# fixture\n").unwrap();
        change::write_default_policy(root, vec!["true".to_string()]).expect("policy");
        git_in(root, &["add", "."]);
        git_in(root, &["commit", "-m", "base"]);
    }

    /// Drives one change to `Verifying` with committed verification and a current scoped
    /// review — the exact state `ship-status` claims is ready and `finalize` is asked to
    /// honour. The caller owns the repository and the branch, because the three #743/#689
    /// cases differ only in what happens to history afterwards.
    fn reviewed_change_fixture(root: &Path) -> String {
        let record = draft_fixture(root);
        let id = record.id.clone();
        for (question, answer) in [
            ("acceptance_criteria", "the fixture verifies"),
            ("public_contract", "no"),
            ("architecture_risk", "no"),
        ] {
            change::answer_question(root, &id, question, answer).expect("answer");
        }
        // Fill whatever artifacts the adaptive interview selected; approval refuses on any
        // scaffold left incomplete.
        let workspace = root.join(".specsync/changes").join(&id);
        for entry in fs::read_dir(&workspace).expect("workspace") {
            let path = entry.expect("entry").path();
            if path.extension().and_then(|e| e.to_str()) == Some("md")
                && path.file_name().and_then(|n| n.to_str()) != Some("change.md")
            {
                let artifact = path.file_stem().unwrap().to_string_lossy().to_string();
                fs::write(
                    &path,
                    format!("---\nchange: {id}\nartifact: {artifact}\n---\n\n# {artifact}\n\nFilled by the fixture.\n"),
                )
                .expect("fill artifact");
            }
        }
        change::approve_definition(root, &id, Some("Reviewer".into()), None).expect("approve");
        change::start_implementation(root, &id).expect("implement");
        git_in(root, &["add", "."]);
        git_in(root, &["commit", "-m", "implement"]);
        // `check_change`, not `verify_change`: the CLI's `change check` materializes the
        // approved deltas as well as verifying, and `finalize` refuses outright on a change
        // whose canonical deltas were never applied. A fixture that only verifies would make
        // `finalize` fail for a reason that has nothing to do with the review — exactly the
        // kind of accidental agreement these tests must not be built on.
        change::check_change(root, Some(&id)).expect("check");
        git_in(root, &["add", "."]);
        git_in(root, &["commit", "-m", "record verification"]);
        change::record_scoped_review_with_verdict(
            root,
            &id,
            "Independent".into(),
            change::ScopedReviewVerdict::Pass,
        )
        .expect("review");
        id
    }

    /// Honest label: CONTROL for the two tests below, and it passes on the unfixed binary
    /// too — that is exactly why it is here.
    ///
    /// #743 asks readiness to consult review currency, and the cheap way to satisfy any test
    /// that demands `ready_to_finalize: false` is to make the conjunction stricter until
    /// nothing is ever ready. This pins the other side: a change whose review was recorded
    /// against this very tree, with nothing touched since, must still reach
    /// `ready_to_finalize: true`.
    ///
    /// What breaks this test: adding any further term to the `ready_to_finalize` conjunction
    /// that a healthy, just-reviewed change cannot satisfy — most obviously a bare
    /// `&& scoped_review_current` implemented as "the descendant walk must have proven
    /// something", since on a squash-merging repository that walk is unavailable rather than
    /// satisfied and the whole class of changes goes permanently red. That is the failure
    /// #689 removed from the verification half, and trading a false green for a permanent
    /// false red is not a fix.
    #[test]
    fn ship_status_is_ready_when_the_scoped_review_is_current() {
        let temp = TempDir::new().expect("temp project");
        let root = temp.path();
        git_project_fixture(root);

        let id = reviewed_change_fixture(root);
        let record = change::load_change(root, &id).expect("reload");
        let report = ship_status_report(root, &record).expect("ship status");

        assert_eq!(
            report["ready_to_finalize"], true,
            "a healthy just-reviewed change must still be ready to finalize: {report}"
        );
        let blockers = report["blockers"].as_array().expect("blockers");
        assert!(
            blockers.is_empty(),
            "a current review must raise no blocker: {blockers:?}"
        );
    }

    /// Honest label: TRUE DISCRIMINATOR for #743.
    ///
    /// `ready_to_finalize` asked `review_path.is_file()` — existence — while `finalize`
    /// additionally required `scoped_review_is_current`. So ship-status recommended
    /// `specsync change ship` and `finalize` refused it one command later, with only a
    /// read-only command in between.
    ///
    /// The staleness here is genuine and by CONTENT: the implementation changes after the
    /// review is recorded, and verification is then re-run so that the verification half of
    /// the conjunction is healthy again. That isolation matters — every content digest the
    /// review checks is also checked by verification, so without the re-check the
    /// verification blocker alone would make readiness false and the test would prove
    /// nothing.
    ///
    /// Discriminates: on a binary built from unfixed `main` this fails with
    /// `ready_to_finalize` = true while `finalize_change` returns
    /// `Err("independent scoped review is stale; ...")`.
    ///
    /// The assertion is AGREEMENT between the two commands, not `ready_to_finalize == false`.
    /// #694 has three live options for the unavailable case and all three end with these two
    /// agreeing; gating on a particular value would go red the day #694 lands the other way.
    #[test]
    fn ship_status_and_finalize_agree_when_the_review_is_stale_by_content() {
        let temp = TempDir::new().expect("temp project");
        let root = temp.path();
        git_project_fixture(root);

        let id = reviewed_change_fixture(root);

        // The implementation genuinely moves after the review, and verification catches up.
        fs::write(root.join("README.md"), "# fixture\n\nsecond thoughts\n").unwrap();
        git_in(root, &["add", "."]);
        git_in(
            root,
            &["commit", "-m", "change the implementation after the review"],
        );
        change::check_change(root, Some(&id)).expect("re-check");
        git_in(root, &["add", "."]);
        git_in(root, &["commit", "-m", "record fresh verification"]);

        let record = change::load_change(root, &id).expect("reload");
        let report = ship_status_report(root, &record).expect("ship status");
        assert_eq!(
            report["verification_present"], true,
            "premise: verification itself is healthy, so only the review half is in question: {report}"
        );

        let ready = report["ready_to_finalize"] == serde_json::Value::Bool(true);
        let finalize = change::finalize_change(root, &id);

        assert_eq!(
            ready,
            finalize.is_ok(),
            "ship-status and finalize disagree about the same change in the same second: ready_to_finalize={ready}, finalize={finalize:?}, report={report}"
        );

        // Agreement is worth nothing if the two commands agree by accident on unrelated
        // grounds, so the refusal `finalize` produced has to be the scoped-review one this
        // test is about. Unconditional here, unlike the squash case: a content change after
        // the review is a decided negative under every option #694 has on the table.
        assert!(
            finalize
                .as_ref()
                .err()
                .is_some_and(|error| error.contains("independent scoped review is stale")),
            "the agreement must be about the review, not some other refusal: {finalize:?}"
        );
        // A decided negative has to name what moved, or the reader is told "not ready" and
        // left to guess (#689's content framing applied to the review half).
        assert_eq!(
            report["review_currency"].as_str(),
            Some("stale"),
            "a content change after the review is a DECIDED negative, not an unavailable one: {report}"
        );
        let blockers: Vec<&str> = report["blockers"]
            .as_array()
            .map(|values| values.iter().filter_map(|value| value.as_str()).collect())
            .unwrap_or_default();
        assert!(
            blockers
                .iter()
                .any(|blocker| blocker.contains("scoped review is stale")),
            "readiness must name the stale review it is refusing on: {blockers:?}"
        );
    }

    /// Honest label: DISCRIMINATOR for #689, relabelled by #743.
    ///
    /// #689: a squash-merge rewrites the recorded verification commit, so
    /// `merge-base --is-ancestor` can never hold again — and this repository, like most,
    /// permits only squash merges. Before that fix `ready_to_finalize` required the
    /// ancestry, so a squash-merged change was permanently unfinalizable while its evidence
    /// was perfectly good. That half is asserted unchanged below: no verification blocker,
    /// and `verification_current` still true.
    ///
    /// What #743 changed is the OTHER half. This test used to assert `ready_to_finalize:
    /// true` here, which was the defect written down as an expectation: `finalize` refuses on
    /// this exact tree with "independent scoped review is stale", because the squash also
    /// destroyed `review.implementation_commit` and the descendant walk cannot run. The
    /// guarantee is unavailable, not violated, and #694 states the standard — "an unavailable
    /// guarantee reported as a satisfied one is worse than the current failure".
    ///
    /// So the assertion is AGREEMENT, exactly as in the stale case, and it survives whichever
    /// way #694 is resolved: if the walk gains a content fallback, `finalize` starts
    /// succeeding and readiness follows it back to `true` through the same predicate.
    #[test]
    fn ship_status_and_finalize_agree_after_a_squash_that_preserves_content() {
        let temp = TempDir::new().expect("temp project");
        let root = temp.path();
        git_project_fixture(root);
        git_in(root, &["switch", "-c", "feature"]);

        let id = reviewed_change_fixture(root);

        // Squash onto main: content identical, recorded commits unreachable. Exactly what
        // GitHub does, and the only strategy many repositories permit.
        git_in(root, &["switch", "main"]);
        git_in(root, &["merge", "--squash", "feature"]);
        git_in(root, &["commit", "-m", "squash feature"]);

        let record = change::load_change(root, &id).expect("reload");
        let report = ship_status_report(root, &record).expect("ship status");

        assert_eq!(
            report["verification_ancestor_of_head"], false,
            "the premise: the squash destroyed the recorded commit"
        );

        // #689, untouched: the verification half is a content question and the content is
        // intact, so nothing about verification may block.
        let blockers: Vec<&str> = report["blockers"]
            .as_array()
            .map(|values| values.iter().filter_map(|value| value.as_str()).collect())
            .unwrap_or_default();
        assert!(
            !blockers
                .iter()
                .any(|blocker| blocker.contains("verification")),
            "a preserved-content squash must raise no verification blocker: {blockers:?}"
        );

        let ready = report["ready_to_finalize"] == serde_json::Value::Bool(true);
        let finalize = change::finalize_change(root, &id);
        assert_eq!(
            ready,
            finalize.is_ok(),
            "ship-status and finalize disagree about the same change in the same second: ready_to_finalize={ready}, finalize={finalize:?}, report={report}"
        );

        // Guards against agreeing by accident, and phrased so that #694 resolving the other
        // way clears it rather than reddening it: IF finalize refused, the refusal has to be
        // the scoped-review one.
        if let Err(error) = &finalize {
            assert!(
                error.contains("independent scoped review is stale"),
                "the agreement must be about the review, not some other refusal: {error}"
            );
        }

        // CHARACTERIZATION of #694's open state, not part of the gate. Today the walk is
        // unobtainable after a squash and readiness says so instead of rounding it to either
        // of the other two answers. If #694 lands a content fallback, this line becomes
        // `current` and should be updated — the agreement assertion above should not.
        assert_eq!(
            report["review_currency"].as_str(),
            Some("unavailable"),
            "a squash makes the descendant walk unobtainable, not violated: {report}"
        );
    }

    /// The ship lane may narrow the next action; it may never contradict the
    /// lifecycle state (#534). At draft the lane wanted `change check --commit`
    /// while the change was still in its interview, so obeying the printed line
    /// failed against the same binary that printed it.
    #[test]
    fn draft_ship_next_defers_to_the_lifecycle_next_action() {
        let temp = TempDir::new().expect("temp project");
        let root = temp.path();
        let record = draft_fixture(root);

        let report = ship_status_report(root, &record).expect("ship status");
        let ship_next = report["ship_next"].as_str().expect("ship_next");
        let lifecycle_next = report["lifecycle_next"].as_str().expect("lifecycle_next");

        assert_eq!(ship_next, lifecycle_next, "{report}");
        assert!(
            !ship_next.contains("--commit"),
            "a draft must not be told to commit verification: {ship_next}"
        );
    }

    /// `ship_next` must name a runnable action, never restate a blocker. Blockers
    /// already render on their own `Blocker:` lines, so repeating one here cost the
    /// reader their next step and bought nothing (#534).
    ///
    /// Honest label: this is an INVARIANT, not a discriminator. It passes on the
    /// unfixed binary too, because a draft carries no blockers and the defect lives at
    /// `Approved` — which this fixture cannot reach without driving a full interview and
    /// approval. Drill 053's approved-state gate is what actually judges that half; this
    /// guards against a future regression reintroducing the restatement anywhere.
    #[test]
    fn ship_next_is_an_action_never_a_blocker_restatement() {
        let temp = TempDir::new().expect("temp project");
        let root = temp.path();
        let record = draft_fixture(root);

        let report = ship_status_report(root, &record).expect("ship status");
        let ship_next = report["ship_next"].as_str().expect("ship_next");
        let blockers: Vec<&str> = report["blockers"]
            .as_array()
            .map(|values| values.iter().filter_map(|v| v.as_str()).collect())
            .unwrap_or_default();

        for blocker in blockers {
            assert_ne!(
                ship_next, blocker,
                "Next: restated a blocker instead of naming an action: {report}"
            );
        }
    }

    /// Evidence must be resolved from wherever the change lives. Both reads used a
    /// hard-coded `.specsync/changes/<id>/` — a parallel implementation of
    /// `change_dir` that an archived change has moved out of — so a finalized change
    /// reported `Verification: none` / `Review: missing` for artifacts sitting in its
    /// own archive package (#534).
    #[test]
    fn archived_evidence_is_resolved_from_the_archive_package() {
        let temp = TempDir::new().expect("temp project");
        let root = temp.path();
        let record = draft_fixture(root);

        // Move the workspace to the archive exactly as finalize does, leaving the
        // active path empty, then teach the record it is archived.
        let active = root.join(".specsync/changes").join(&record.id);
        let archived = root
            .join(".specsync/archive/changes")
            .join(format!("2026-08-19-{}", record.id));
        fs::create_dir_all(archived.parent().expect("archive parent")).expect("archive root");
        fs::rename(&active, &archived).expect("archive the workspace");
        fs::write(
            archived.join("review.json"),
            "{\"reviewer\":\"peer\",\"verdict\":\"pass\"}\n",
        )
        .expect("write review evidence");
        fs::write(
            archived.join("verification.json"),
            format!("{{\"commit\":\"{}\",\"passed\":true}}\n", "a".repeat(40)),
        )
        .expect("write verification evidence");
        let mut archived_record = record.clone();
        archived_record.state = ChangeState::Archived;
        fs::write(
            archived.join("state.json"),
            serde_json::to_string(&archived_record).expect("serialize archived state"),
        )
        .expect("write archived state");

        let report = ship_status_report(root, &archived_record).expect("ship status");

        assert_eq!(
            report["verification_commit"].as_str(),
            Some("a".repeat(40).as_str()),
            "archived verification was not resolved: {report}"
        );
        assert_eq!(
            report["review_present"].as_bool(),
            Some(true),
            "archived review was not resolved: {report}"
        );
    }

    /// The read must be lenient. A strict `?` on the archived artifact turns
    /// `ship-status` and `ship` from rc=0 into rc=1 on a repository whose evidence is
    /// already damaged — the fix for an inspection command must not be the thing that
    /// breaks inspection (#534).
    #[test]
    fn a_corrupt_archived_verification_degrades_instead_of_failing() {
        let temp = TempDir::new().expect("temp project");
        let root = temp.path();
        let record = draft_fixture(root);

        let active = root.join(".specsync/changes").join(&record.id);
        let archived = root
            .join(".specsync/archive/changes")
            .join(format!("2026-08-19-{}", record.id));
        fs::create_dir_all(archived.parent().expect("archive parent")).expect("archive root");
        fs::rename(&active, &archived).expect("archive the workspace");
        fs::write(archived.join("verification.json"), "not json at all\n")
            .expect("write corrupt evidence");
        let mut archived_record = record.clone();
        archived_record.state = ChangeState::Archived;
        fs::write(
            archived.join("state.json"),
            serde_json::to_string(&archived_record).expect("serialize archived state"),
        )
        .expect("write archived state");

        let report = ship_status_report(root, &archived_record)
            .expect("a corrupt archived artifact must not fail the command");

        assert!(
            report["verification_commit"].is_null(),
            "corrupt evidence must read as absent, not as a commit: {report}"
        );
    }

    // Verifies REQ-cmd-change-009.
    #[test]
    fn text_correction_ledger_gate_uses_a_safe_diagnostic() {
        let temp = TempDir::new().expect("temp project");
        let root = temp.path();
        change::write_default_policy(root, Vec::new()).expect("write default policy");
        let record = change::create_change(
            root,
            CreateChangeRequest {
                description: "Protect text lifecycle views".into(),
                kind: ChangeKind::Documentation,
                affected_specs: Vec::new(),
                affected_paths: vec!["README.md".into()],
                requested_artifacts: Vec::new(),
                no_spec_change: true,
                rationale: Some("Command test fixture".into()),
            },
        )
        .expect("create draft");
        let ledger_path = root
            .join(".specsync/changes")
            .join(&record.id)
            .join("corrections.json");
        fs::write(&ledger_path, "{ malformed correction ledger\\n")
            .expect("write malformed ledger");

        let error = ensure_text_correction_ledger_valid(root, &record).expect_err("invalid ledger");

        assert_eq!(error, INVALID_CORRECTION_LEDGER_TEXT);
        assert!(!error.contains("malformed correction ledger"));
    }

    // Verifies REQ-cmd-change-010.
    #[test]
    fn mutation_output_uses_the_in_transaction_correction_snapshot() {
        let temp = TempDir::new().expect("temp project");
        let root = temp.path();
        change::write_default_policy(root, Vec::new()).expect("write default policy");
        let record = change::create_change(
            root,
            CreateChangeRequest {
                description: "Render one successful lifecycle mutation".into(),
                kind: ChangeKind::Documentation,
                affected_specs: Vec::new(),
                affected_paths: vec!["README.md".into()],
                requested_artifacts: Vec::new(),
                no_spec_change: true,
                rationale: Some("Command test fixture".into()),
            },
        )
        .expect("create draft");
        let result = change::answer_question_with_snapshot(
            root,
            &record.id,
            "acceptance_criteria",
            "The successful mutation renders from its validated transaction snapshot",
        )
        .expect("persist mutation");
        let ledger_path = root
            .join(".specsync/changes")
            .join(&record.id)
            .join("corrections.json");
        fs::write(&ledger_path, "{ malformed correction ledger\\n")
            .expect("corrupt ledger after persistence");

        assert!(change::effective_change_definition(root, &result.change).is_err());
        assert!(!change::summarize_change(root, &result.change).correction_valid);
        assert!(result.summary.correction_valid);
        assert!(result.strict_summary.correction_valid);
        assert!(
            print_mutation_record(root, &record.id, &result, OutputFormat::Text, true, false)
                .is_ok()
        );
        assert!(
            print_mutation_record(root, &record.id, &result, OutputFormat::Json, true, false)
                .is_ok()
        );
    }

    #[test]
    fn classify_paths_distinguishes_product_review_and_archive_tips() {
        assert_eq!(classify_paths(&[]), "other");
        assert_eq!(
            classify_paths(&["src/commands/change.rs".into()]),
            "product"
        );
        assert_eq!(
            classify_paths(&[
                ".specsync/changes/CHG-0001-demo/review.json".into(),
                ".specsync/changes/CHG-0001-demo/review-attempts.json".into(),
            ]),
            "review_only"
        );
        assert_eq!(
            classify_paths(&[
                ".specsync/archive/changes/2026-08-07-CHG-0001-demo/state.json".into(),
                ".specsync/archive/changes/2026-08-07-CHG-0001-demo/finalization.json".into(),
            ]),
            "archive_only"
        );
        assert_eq!(
            classify_paths(&[
                "src/cli.rs".into(),
                ".specsync/changes/CHG-0001-demo/review.json".into(),
            ]),
            "product"
        );
    }

    /// #687: the warning must name the cost that lands on OTHER changes, not only this one.
    /// Pins the disclosure so a future refactor cannot quietly shrink it back to "strands the
    /// change", which under-prices the decision it exists to inform.
    #[test]
    fn the_merge_warning_names_the_cost_to_earlier_accepted_changes() {
        for still_active in [false, true] {
            let warning = merge_before_finalize_warning(still_active);
            assert!(
                warning.contains("every earlier accepted change sharing a delivery input"),
                "must name whose work is blocked: {warning}"
            );
            assert!(
                warning.contains("from archiving"),
                "must name what is blocked: {warning}"
            );
            assert!(
                warning.contains("finalized or those are reopened"),
                "must name both exits: {warning}"
            );
        }
        assert!(merge_before_finalize_warning(false).contains("before finalize"));
        assert!(merge_before_finalize_warning(true).contains("still active"));
    }

    #[test]
    fn multi_active_ordering_warnings_encode_four_rules() {
        assert!(multi_active_ordering_warnings(&[]).is_empty());
        let warnings = multi_active_ordering_warnings(&["CHG-0002-sibling".into()]);
        assert_eq!(warnings.len(), 3);
        let joined = warnings.join(" ");
        assert!(joined.contains("finalize one at a time"), "{joined}");
        assert!(joined.contains("do not batch reviews"), "{joined}");
        assert!(joined.contains("still active"), "{joined}");
        assert!(joined.contains("CHG-0002-sibling"), "{joined}");
    }
}

/// Perform the verify-commit-verify-commit sequence `change check` otherwise
/// requires an author to remember.
///
/// `change check` materializes the approved delta into the working tree and
/// anchors verification at the commit that predates it. Committing therefore
/// stales the evidence just recorded, and the only way to reach a state CI
/// accepts is to verify again against the committed tree. Nothing said so, and
/// the failure surfaced only in CI — as four red checks reporting one cause.
///
/// SpecSync still does not commit by default; this runs only under `--commit`.
///
/// Nothing is committed unless verification passes: a half-committed lifecycle is
/// worse than none.
fn run_checked_commit(
    root: &std::path::Path,
    id: Option<&str>,
    strict: bool,
    push: bool,
    format: OutputFormat,
) -> Result<(), String> {
    let quiet = matches!(format, OutputFormat::Json);
    let say = |message: &str| {
        if !quiet {
            println!("{message}");
        }
    };

    let verify = |label: &str| -> Result<Option<String>, String> {
        say(label);
        let record = if strict {
            change::check_change_with_strict(root, id, true)?
        } else {
            change::check_change(root, id)?
        };
        Ok(record.map(|record| record.commit.unwrap_or_default()))
    };

    // First pass: materialize the approved delta and verify the working tree.
    verify("Checking (1/2): verifying the materialized tree…")?;

    let resolved = match id.map(str::to_string) {
        Some(id) => id,
        None => {
            // This writes a commit, so it must not guess from a partial roster:
            // the change it is looking for may be the very one it could not read.
            let roster = change::list_changes(root)?;
            if let Some(unreadable) = roster.unreadable.first() {
                return Err(format!(
                    "cannot resolve a change to commit while a workspace is unreadable: {}",
                    unreadable.reason
                ));
            }
            roster
                .records
                .into_iter()
                .find(|record| record.canonical_applied)
                .map(|record| record.id)
                .ok_or_else(|| "cannot resolve a change to commit".to_string())?
        }
    };

    git_commit_all(root, &format!("chore(lifecycle): materialize {resolved}"))?;
    say("Committed the materialized spec.");

    // Second pass: re-anchor against the committed tree. Evidence files live under
    // `.specsync/changes/`, which is excluded from the project-input digest, so
    // committing them cannot stale this result.
    verify("Checking (2/2): re-verifying against the committed tree…")?;
    git_commit_all(
        root,
        &format!("chore(lifecycle): record {resolved} verification"),
    )?;
    say("Committed the verification evidence.");

    if push {
        run_git(root, &["push"])?;
        say("Pushed.");
    }
    say("Verified, committed, and consistent with the committed tree.");
    Ok(())
}

fn run_git(root: &std::path::Path, args: &[&str]) -> Result<(), String> {
    let output = std::process::Command::new("git")
        .args(args)
        .current_dir(root)
        .output()
        .map_err(|error| format!("failed to run git {}: {error}", args.join(" ")))?;
    if !output.status.success() {
        return Err(format!(
            "git {} failed: {}",
            args.join(" "),
            String::from_utf8_lossy(&output.stderr).trim()
        ));
    }
    Ok(())
}

/// Stage everything and commit, treating "nothing to commit" as success so the
/// sequence stays idempotent when a pass produced no change.
fn git_commit_all(root: &std::path::Path, message: &str) -> Result<(), String> {
    // Every lifecycle commit stages `-A`, so every one of them can commit a
    // sequence ledger that went stale while the branch sat. Flooring here rather
    // than at the single call site named in the report covers all three, which is
    // the difference between fixing this and fixing where it was noticed.
    // Reported on stderr rather than through the caller's `say`: this is a state
    // correction, not progress chatter, so it must survive `--quiet`, and it must
    // stay off stdout where a `--format json` payload is being written.
    if let Some((was, now)) = change::floor_sequence_ledger_to_committed(root)? {
        eprintln!(
            "note: raised the change sequence ledger from {was} to the committed {now} before staging; \
             a ledger written before the branch caught up would have committed a lower high-water mark"
        );
    }
    run_git(root, &["add", "-A"])?;
    let status = std::process::Command::new("git")
        .args(["diff", "--cached", "--quiet"])
        .current_dir(root)
        .status()
        .map_err(|error| format!("failed to inspect staged changes: {error}"))?;
    if status.success() {
        return Ok(());
    }
    run_git(root, &["commit", "-m", message])
}

#[cfg(test)]
mod ship_next_action_tests {
    use super::*;

    // Honest label: DISCRIMINATOR. Lessons are a convention, not a merge gate.
    // On the unfixed binary this string started with "write lessons into…".
    #[test]
    fn ship_does_not_gate_merge_on_writing_lessons() {
        let next = ship_next_action(
            true,
            true,
            &[],
            &["specs/change/context.md".to_string()],
            "/archive/lesson-bundle.md",
        );

        assert_eq!(next, "merge the PR on GitHub when Required CI is green");
        assert!(!next.contains("write lessons"), "got {next:?}");
    }

    // Honest label: this is the CONTROL. A change owning no specs has nothing to fold, and its
    // guidance must be BYTE-IDENTICAL to the guidance before this fix — so the expected strings
    // are spelled out rather than probed for the absence of "write lessons", which would still
    // pass if the tail were reworded. All four push/wait combinations, including `(false, true)`:
    // `--wait` is independent of `--push` in the CLI, so that arm is reachable.
    #[test]
    fn ship_guidance_is_unchanged_when_there_is_nothing_to_fold() {
        let cases = [
            (
                true,
                true,
                "merge the PR on GitHub when Required CI is green",
            ),
            (
                true,
                false,
                "wait for CI, then merge the PR (or re-run `change ship --wait`)",
            ),
            // `--wait` without `--push` collapses into the no-push branch, so the guidance
            // still says to push the archive tip. That is pre-existing behaviour, unchanged
            // here, and pinning it is the point of a control: this test exists to detect any
            // drift in these strings, not to endorse them.
            (
                false,
                true,
                "commit if needed, push the archive tip, wait for CI, then merge the PR",
            ),
            (
                false,
                false,
                "commit if needed, push the archive tip, wait for CI, then merge the PR",
            ),
        ];

        for (push, wait, expected) in cases {
            assert_eq!(
                ship_next_action(push, wait, &[], &[], "/archive/lesson-bundle.md"),
                expected
            );
        }
    }

    // Sibling changes still block the merge, and that warning must survive the prefix rather
    // than being displaced by it.
    #[test]
    fn ship_keeps_the_sibling_blocker_alongside_the_fold_back() {
        let next = ship_next_action(
            false,
            false,
            &["other-change".to_string()],
            &["specs/cmd_change/context.md".to_string()],
            "/archive/lesson-bundle.md",
        );

        assert!(
            !next.contains("write lessons"),
            "lessons must not displace the sibling blocker: {next:?}"
        );
        assert!(next.contains("do not merge while any change is active"));
        assert!(next.contains("other-change"));
    }
}
