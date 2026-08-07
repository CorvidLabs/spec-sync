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
        ChangeAction::ShipStatus { id } => {
            let _scope = change::begin_change_read_scope(root);
            if let Some(id) = id {
                change::load_change(root, &id)
                    .and_then(|record| print_ship_status(root, &record, format))
            } else {
                let records = change::list_changes(root);
                if records.is_empty() {
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
                            print_json(&serde_json::json!({ "changes": reports }));
                        })
                } else {
                    records.iter().try_for_each(|record| {
                        print_ship_status(root, record, format)?;
                        println!();
                        Ok(())
                    })
                }
            }
        }
        ChangeAction::Ship { id, dry_run } => run_ship(root, id.as_deref(), dry_run, format),
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
                        let verified_id = id.clone().or_else(|| {
                            change::list_changes(root)
                                .into_iter()
                                .find(|record| {
                                    matches!(
                                        record.state,
                                        ChangeState::Implementing | ChangeState::Verifying
                                    )
                                })
                                .map(|record| record.id)
                        });
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
            // (CodeQL rust/cleartext-logging). Human output uses interview/state only.
            let id = record.id.clone();
            let title = record.title.clone();
            let state = record.state.as_str().to_owned();
            println!("{} {}", id.bold(), title);
            println!("  State: {state}");
            println!(
                "  Next: {}",
                text_mode_next_action(root, record, &questions)
            );
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
                "run `specsync change check {id} --commit` if needed, then `specsync change ship-status {id}` (or `ship {id}`) — independent review then finalize before merging; merging first orphans verification evidence"
            )
        }
        ChangeState::Accepted if record.workflow_version >= 2 => {
            format!("run `specsync change ship {id}` or `specsync change finalize {id}`")
        }
        ChangeState::Accepted => format!("run `specsync change archive {id}`"),
        ChangeState::Archived => "no further action".into(),
    }
}

/// Ship readiness for one change: HEAD tip class, verification tip health, review,
/// trust guidance for staged product → review → archive tips, and the merge-before-finalize trap.
/// Does not query GitHub check-runs (local/offline safe).
fn ship_status_report(root: &Path, record: &ChangeRecord) -> Result<serde_json::Value, String> {
    let questions = change::next_questions(record);
    let lifecycle_next = text_mode_next_action(root, record, &questions);
    let tip = classify_head_tip(root)?;

    let verification_path = root
        .join(".specsync/changes")
        .join(&record.id)
        .join("verification.json");
    let (verification_commit, verification_present, verification_ancestor) = if verification_path
        .is_file()
    {
        let raw = fs::read_to_string(&verification_path)
            .map_err(|error| format!("failed to read {}: {error}", verification_path.display()))?;
        let value: serde_json::Value = serde_json::from_str(&raw)
            .map_err(|error| format!("invalid verification.json for {}: {error}", record.id))?;
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

    let review_path = root
        .join(".specsync/changes")
        .join(&record.id)
        .join("review.json");
    let review_present = review_path.is_file();

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
        } else if !verification_ancestor {
            blockers.push(
                "verification commit is not an ancestor of HEAD; re-run change check --commit before review/finalize"
                    .to_string(),
            );
        }
        if record.state == ChangeState::Verifying && !review_present {
            warnings.push(
                "no scoped review recorded yet; finalize requires independent review".to_string(),
            );
        }
        if record.state == ChangeState::Verifying {
            warnings.push(
                "do not merge the PR before finalize — merging first orphans verification evidence and strands the change"
                    .to_string(),
            );
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

    let ready_to_finalize = record.state == ChangeState::Verifying
        && verification_present
        && verification_ancestor
        && review_present
        && blockers.is_empty();

    let stages = ship_stages(
        record,
        &tip,
        verification_commit.as_deref(),
        verification_present,
        verification_ancestor,
        review_present,
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
        format!(
            "run `specsync change ship {}` (or finalize without intermediate commits), push the archive tip, wait for CI, then merge the PR",
            record.id
        )
    } else if !blockers.is_empty() {
        blockers[0].clone()
    } else {
        current_stage
            .get("action")
            .and_then(|value| value.as_str())
            .unwrap_or(lifecycle_next.as_str())
            .to_string()
    };

    let trust = serde_json::json!({
        "status": "local_guidance",
        "parent_sha": tip.parent_sha,
        "guidance": "After pushing a product tip, wait for trust + SpecSync implementation ready (and required product checks) to succeed before pushing a review_only or archive_only tip. Review/archive tips reuse trust from a green product parent — do not push them while product trust is still running or cancelled.",
    });

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
        "ready_to_finalize": ready_to_finalize,
        "blockers": blockers,
        "warnings": warnings,
        "lifecycle_next": lifecycle_next,
        "ship_next": ship_next,
    }))
}

fn ship_stages(
    record: &ChangeRecord,
    tip: &HeadTip,
    verification_commit: Option<&str>,
    verification_present: bool,
    verification_ancestor: bool,
    review_present: bool,
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

    let review_status = if review_present {
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
        "action": if review_present {
            "scoped review is recorded".to_string()
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
            println!(
                "  Review: {}",
                if report["review_present"].as_bool() == Some(true) {
                    "recorded"
                } else {
                    "missing"
                }
            );
            if let Some(guidance) = report
                .pointer("/trust/guidance")
                .and_then(|value| value.as_str())
            {
                println!("  Trust: {guidance}");
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
    format: OutputFormat,
) -> Result<(), String> {
    let record =
        match id {
            Some(id) => change::load_change(root, id)?,
            None => {
                let records = change::list_changes(root);
                match records.as_slice() {
                    [] => return Err("no active change to ship; pass an explicit change id".into()),
                    [single] => single.clone(),
                    _ => return Err(
                        "multiple active changes; pass an explicit id: `specsync change ship <ID>`"
                            .into(),
                    ),
                }
            }
        };

    if record.state == ChangeState::Archived {
        let report = ship_status_report(root, &record)?;
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

    // Finalize mutates the workspace; drop the read scope by ending this block first.
    let path = change::finalize_change(root, &record.id)?;
    match format {
        OutputFormat::Json => print_json(&serde_json::json!({
            "id": record.id,
            "status": "finalized",
            "archived": path,
            "tip_class": "archive_only",
            "next": "commit if needed, push the archive tip, wait for CI, then merge the PR",
        })),
        _ => {
            println!("{} {} finalized on this PR", "✓".green(), record.id);
            println!("  Archive: {}", path.display());
            println!(
                "  Next: push this archive tip, wait for CI (archive_only reuses product trust), then merge the PR — do not merge before finalize"
            );
        }
    }
    Ok(())
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
                let next = text_mode_next_action(root, record, &questions);
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

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

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

    let resolved = id
        .map(str::to_string)
        .or_else(|| {
            change::list_changes(root)
                .into_iter()
                .find(|record| record.canonical_applied)
                .map(|record| record.id)
        })
        .ok_or_else(|| "cannot resolve a change to commit".to_string())?;

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
