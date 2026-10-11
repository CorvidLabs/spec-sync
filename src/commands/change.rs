use self::archive_guidance::{
    archive_sentence_without_command, lessons_next_action, pull_request_sentence,
};
use self::lifecycle_commit::{disclose_left_out, git_commit_lifecycle, run_checked_commit};
use self::owner_inputs::resolve_correct_owner_entries;
use self::render::{
    lesson_bundle_line, print_handoff_line, print_handoff_line_for, print_json,
    print_mutation_record, print_record, print_roster, print_transition,
    print_transition_with_handoff, print_unreadable_rows, text_mode_next_action, unreadable_error,
    unreadable_json,
};
use self::ship_status::{print_ship_status, ship_status_report};
use self::shipping::{run_prepare_pr, run_ship};
use crate::change;
use crate::change::ArtifactKind;
use crate::change::ChangeKind;
use crate::change::CorrectionField;
use crate::change::CreateChangeRequest;
use crate::cli::ChangeAction;
use crate::types::OutputFormat;
use colored::Colorize;
use std::fs;
use std::path::Path;
use std::process;

mod archive_guidance;
mod lifecycle_commit;
mod owner_inputs;
mod render;
mod repo_status;
mod ship_status;
mod shipping;

#[cfg(test)]
mod ship_next_action_tests;
#[cfg(test)]
mod tests;

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
            // A broken policy must not look like "no principles". `rules` already
            // fails here; status was still printing a briefing as if the file parsed.
            if let Err(error) = change::configured_principles_file(root) {
                Err(error)
            } else {
                let _scope = change::begin_change_read_scope(root);
                if !matches!(format, OutputFormat::Json) {
                    match pull_request_sentence(root) {
                        Ok(sentence) => {
                            println!("{}", archive_sentence_without_command(&sentence))
                        }
                        Err(error) => eprintln!("{} {error}", "error:".red().bold()),
                    }
                }
                if let Some(id) = id {
                    change::load_change(root, &id)
                        .and_then(|record| print_record(root, &record, format, false, strict))
                } else {
                    change::list_changes(root)
                        .and_then(|roster| print_roster(root, &roster, format, strict))
                }
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
        ChangeAction::Pr { id } => run_prepare_pr(root, id.as_deref(), format),
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
            true,
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
            result.map(|record| {
                print_transition_with_handoff(root, &record, format, "definition approved")
            })
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
                    _ => {
                        println!(
                            "{} {} scoped review recorded as {} at {}",
                            "✓".green(),
                            review.change_id,
                            review.verdict.as_str(),
                            review.implementation_commit
                        );
                        print_handoff_line_for(root, &review.change_id);
                    }
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
        ChangeAction::Finalize { id } => {
            // A commit that fails after the move leaves the change archived and
            // the package uncommitted. Finalize again must commit that package,
            // not refuse because the state is already Archived.
            let archived = change::load_change(root, &id)
                .ok()
                .is_some_and(|record| record.state == change::ChangeState::Archived);
            let path = if archived {
                change::find_change_dir(root, &id)
            } else {
                change::finalize_change(root, &id)
            };
            path.and_then(|path| {
            // The archive move is a working-tree rename until something commits it.
            // Ship already commits. Finalize has to do the same, or the package
            // exists only on disk and the next checkout loses it.
            let archive_commit = git_commit_lifecycle(
                root,
                &change::lifecycle_commit_scope(root, &id)?,
                &format!("chore(lifecycle): archive {id}"),
                true,
            )?;
            disclose_left_out(&archive_commit.left_out, &mut Vec::new(), None);
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
                    println!("  {}", lesson_bundle_line());
                    println!("  Next: {}", lessons_next_action(root, &id, &path));
                    print_handoff_line_for(root, &id);
                }
            }
            Ok(())
        })
        },
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
                                let questions = change::next_questions_for(root, &change_record);
                                println!(
                                    "  Next: {}",
                                    text_mode_next_action(root, &change_record, &questions)
                                );
                                print_handoff_line(&change::handoff_summary(root, &change_record));
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
