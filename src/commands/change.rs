use colored::Colorize;
use std::collections::BTreeSet;
use std::fs;
use std::path::{Path, PathBuf};
use std::process;

use crate::change::{
    self, ArtifactKind, ChangeKind, ChangeRecord, ChangeState, CorrectionField, CreateChangeRequest,
};
use crate::cli::ChangeAction;
use crate::types::OutputFormat;

const INVALID_CORRECTION_LEDGER_TEXT: &str = "correction ledger integrity is invalid; restore corrections.json from trusted history before inspecting lifecycle status";

fn configured_commands_not_run(root: &Path) -> Result<Vec<String>, String> {
    let path = root.join(".specsync/sdd.json");
    let content = match fs::read_to_string(&path) {
        Ok(content) => content,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(Vec::new()),
        Err(error) => return Err(format!("failed to read {}: {error}", path.display())),
    };
    let policy: serde_json::Value = serde_json::from_str(&content)
        .map_err(|error| format!("invalid SDD policy {}: {error}", path.display()))?;
    let mut commands = Vec::new();
    if let Some(configured) = policy["verification_commands"].as_array() {
        commands.extend(configured.iter().filter_map(serde_json::Value::as_str));
    }
    if let Some(components) = policy["component_verification_commands"].as_object() {
        let mut modules = components.keys().collect::<Vec<_>>();
        modules.sort();
        for module in modules {
            if let Some(configured) = components[module].as_array() {
                commands.extend(configured.iter().filter_map(serde_json::Value::as_str));
            }
        }
    }
    if let Some(configured) = policy["strict_verification_commands"].as_array() {
        commands.extend(configured.iter().filter_map(serde_json::Value::as_str));
    }
    let mut seen = BTreeSet::new();
    let mut commands = commands.into_iter().map(str::to_owned).collect::<Vec<_>>();
    commands.retain(|command| seen.insert(command.clone()));
    Ok(commands)
}

fn print_configured_commands_not_run(root: &Path) -> Result<(), String> {
    let commands = configured_commands_not_run(root)?;
    if !commands.is_empty() {
        let noun = if commands.len() == 1 {
            "command"
        } else {
            "commands"
        };
        eprintln!(
            "note: .specsync/sdd.json configures {} verification {noun} that SpecSync did not run; empty verification_commands, component_verification_commands, and strict_verification_commands to stop this note",
            commands.len()
        );
    }
    Ok(())
}

// `specsync doctor` renders here, beside the change commands it reports on (REQ-cmd-change-035).
#[path = "doctor.rs"]
mod doctor;
pub use doctor::cmd_doctor;

pub fn cmd_change(root: &Path, action: ChangeAction, format: OutputFormat, strict: bool) {
    // Repository facts git answers during this command are asked once (REQ-cmd-change-028). Every
    // git write and external process this command runs clears them.
    let _facts = crate::git_utils::begin_repository_fact_scope(root);
    let result = (|| {
        match action {
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
        ChangeAction::ReconcileHistory { id, actor, reason } => {
            change::enable_historical_reconciliation(root, &id, actor, reason).map(|record| {
                print_transition_with_handoff(root, &record, format, "historical reconciliation requested; old file transitions remain unproven; fresh definition approval required")
            })
        }
        ChangeAction::List { timeout } => {
            let _deadline = read_deadline_for("change list", timeout, format);
            let _scope = change::begin_change_read_scope(root);
            change::list_changes(root)
                .and_then(|roster| print_roster(root, &roster, format, strict))
        }
        ChangeAction::Show { id, timeout } => {
            let _deadline = read_deadline_for("change show", timeout, format);
            let _scope = change::begin_change_read_scope(root);
            closed_report(root, &id, format, "change show").unwrap_or_else(|| {
                change::load_change(root, &id)
                    .and_then(|record| print_record(root, &record, format, true, strict))
            })
        }
        ChangeAction::Status { id, timeout } => {
            let _deadline = read_deadline_for("change status", timeout, format);
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
                    closed_late_report(root, &id, format, "change status")
                        .or_else(|| closed_report(root, &id, format, "change status"))
                        .unwrap_or_else(|| {
                        change::load_change(root, &id)
                            .and_then(|record| print_record(root, &record, format, false, strict))
                    })
                } else {
                    change::list_changes(root)
                        .and_then(|roster| print_roster(root, &roster, format, strict))
                }
            }
        }
        ChangeAction::ShipStatus { id, timeout } => {
            let _deadline = read_deadline_for("change ship-status", timeout, format);
            let _scope = change::begin_change_read_scope(root);
            if let Some(id) = id {
                closed_late_report(root, &id, format, "change ship-status")
                    .or_else(|| closed_report(root, &id, format, "change ship-status"))
                    .unwrap_or_else(|| {
                    change::load_change(root, &id)
                        .and_then(|record| print_ship_status(root, &record, format))
                })
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
            words,
            relay_where,
            recorded_by,
        } => change::relayed_decision(
            // The re-run keeps every flag that changes what is recorded (#900 review).
            &format!(
                "specsync change approve {id} --actor <who decided>{}{}",
                if note.is_some() { " --note \"<your note>\"" } else { "" },
                if portable_5_0_1 { " --portable-5-0-1" } else { "" }
            ),
            actor.as_deref(),
            words,
            relay_where,
            recorded_by,
        )
        .and_then(|relayed_by| match relayed_by {
            None if portable_5_0_1 => {
                change::approve_definition_portable_v501(root, &id, actor, note)
            }
            None => change::approve_definition(root, &id, actor, note),
            relayed_by => change::approve_definition_with_relay(
                root,
                &id,
                actor,
                note,
                portable_5_0_1,
                relayed_by,
            ),
        })
        .map(|record| print_transition_with_handoff(root, &record, format, "definition approved")),
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
            words,
            relay_where,
            recorded_by,
        } => change::relayed_decision(
            // The re-run keeps the verdict: dropping `--verdict block` would record a pass (#900
            // review).
            &format!("specsync change review {id} --reviewer <who decided> --verdict {verdict}"),
            Some(reviewer.trim()),
            words,
            relay_where,
            recorded_by,
        )
        .and_then(|relayed_by| {
            change::ScopedReviewVerdict::parse(&verdict).map(|verdict| (verdict, relayed_by))
        })
        .and_then(|(verdict, relayed_by)| {
            let result = match relayed_by {
                None if verdict == change::ScopedReviewVerdict::Pass => {
                    change::record_scoped_review(root, &id, reviewer)
                }
                None => change::record_scoped_review_with_verdict(root, &id, reviewer, verdict),
                relayed_by => {
                    change::record_scoped_review_with_relay(root, &id, reviewer, verdict, relayed_by)
                }
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
        ChangeAction::Retire { id, actor, reason, replacement, authority_source, source_kind } => {
            let result = if source_kind == "accepted-receipt" {
                change::retire_change(root, &id, actor, reason, replacement, authority_source)
            } else {
                change::retire_change_with_source(
                    root, &id, actor, reason, replacement, authority_source,
                    change::RetirementSourceKind::CommittedAuditedReopeningUnproven,
                )
            };
            result.and_then(|record| print_record(root, &record, format, false, strict))
        }
        ChangeAction::SendBack { id, actor, reason } => {
            change::send_back_change(root, &id, actor, reason).map(|result| match format {
                OutputFormat::Json => print_json(&result),
                _ => {
                    println!(
                        "{} {} sent back to draft for re-scoping; its approval is cancelled",
                        "✓".green(),
                        result.change.id
                    );
                    println!(
                        "  Next: answer the scope questions again with `specsync change answer {} <question> <answer>`, then `specsync change approve {} --actor <name>`",
                        result.change.id, result.change.id
                    );
                }
            })
        }
        ChangeAction::Close {
            ids,
            actor,
            reason,
            replaced_by,
            words,
            relay_where,
            recorded_by,
        } => change::relayed_decision(
            &format!(
                "specsync change close {} --actor <who decided> --reason \"<why>\"{}",
                ids.join(" "),
                if replaced_by.is_some() { " --replaced-by <id>" } else { "" }
            ),
            Some(actor.as_str()),
            words,
            relay_where,
            recorded_by,
        )
        .and_then(|relayed_by| {
            change::close_changes(
                root,
                &change::CloseRequest {
                    ids,
                    actor,
                    reason,
                    replaced_by,
                    relayed_by,
                },
            )
        })
        .map(|closed| match format {
            OutputFormat::Json => print_json(&serde_json::json!({ "closed": closed })),
            _ => {
                for change in &closed {
                    println!(
                        "{} {} closed from {}; never reviewed; its package is `{}`",
                        "✓".green(),
                        change.record.change_id,
                        change.record.from_state.as_str(),
                        change.package
                    );
                }
                println!(
                    "  Next: commit the move. If a closed change taught something, record it in the lessons of the modules it touched, as finalize asks"
                );
            }
        }),
        ChangeAction::RepairRelays { id } => {
            change::repair_relayed_decisions(root, &id).map(|report| match format {
                OutputFormat::Json => print_json(&report),
                _ => print_relay_repairs(&report),
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
            let historical = change::load_change(root, &id)?.answers.get("historical_reconciliation").is_some_and(|value| value == "unproven");
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
                }).as_object().map(|object| {
                    let mut object = object.clone();
                    if historical { object.insert("historical_transition_proven".into(), serde_json::json!(false)); }
                    object
                }).unwrap_or_default()),
                _ => {
                    // Digests stay in --json only; text mode names the archive + commit.
                    println!("{} {} finalized on this PR", "✓".green(), id);
                    if historical { println!("  Historical file transitions remain unproven (explicit receipt reconciliation)."); }
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
            if !matches!(format, OutputFormat::Json) {
                print_configured_commands_not_run(root)?;
            }
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
                        print_json(&serde_json::json!({
                            "verification": record,
                            "configured_commands_not_run": configured_commands_not_run(root)?,
                        }));
                        if !record.passed {
                            process::exit(1);
                        }
                    }
                    Ok(None) => {
                        print_json(&serde_json::json!({
                            "verification": null,
                            "message": "nothing to check",
                            "configured_commands_not_run": configured_commands_not_run(root)?,
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
                            print_verification_ignored_lines(root);
                            if let Some(change_id) = &verified_id
                                && let Ok(change_record) = change::load_change(root, change_id)
                            {
                                println!(
                                    "  Next: {}",
                                    text_mode_next_action(root, &change_record)
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
        ChangeAction::Audit { archives } => {
            // `--archives` adds the walk over every archive's own workflow-version history, which
            // finalize no longer repeats (REQ-change-124). `--strict` alone stays as fast as before.
            let report = change::audit_project_with_archives(root, archives);
            let passed = report.errors.is_empty();
            match format {
                OutputFormat::Json => {
                    if passed {
                        print_json(&serde_json::json!({
                            "report": report,
                            "audit_scope": if archives { "active-project-and-archives" } else { "active-project" },
                            "archived_retirement_custody": if archives { "validated" } else { "not-inspected" },
                            "configured_commands_not_run": configured_commands_not_run(root)?,
                        }));
                    } else {
                        print_json(&serde_json::json!({
                            "report": report,
                            "audit_scope": if archives { "active-project-and-archives" } else { "active-project" },
                            "archived_retirement_custody": if archives { "failed-or-incomplete" } else { "not-inspected" },
                        }));
                    }
                }
                _ => {
                    if passed {
                        print_configured_commands_not_run(root)?;
                    }
                    if !archives {
                        println!("Archive history and archived retirement custody were not inspected; use specsync change audit --archives to include them.");
                    }
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
    }
    })();

    if let Err(error) = result {
        match format {
            OutputFormat::Json => print_json(&serde_json::json!({ "error": error })),
            _ => eprintln!("{} {error}", "error:".red().bold()),
        }
        process::exit(1);
    }
}

/// The read deadline for a read-only command, from `--timeout` or `SPECSYNC_TIMEOUT`
/// (REQ-cmd-change-032). The resume command is this invocation without its timeout.
fn read_deadline_for(
    command: &str,
    timeout: Option<u64>,
    format: OutputFormat,
) -> Option<change::ReadDeadlineGuard> {
    let limit = change::read_timeout_secs(timeout)?;
    let resume = resume_command_without_timeout(std::env::args().skip(1));
    Some(change::begin_read_deadline(
        limit,
        command,
        &resume,
        matches!(format, OutputFormat::Json),
    ))
}

/// `specsync` followed by `args` with any `--timeout <secs>` or `--timeout=<secs>` removed, quoted
/// so that a POSIX shell splits it back into exactly those arguments. When `SPECSYNC_TIMEOUT` is
/// set, the command clears it, so the resumed run has no deadline (REQ-cmd-change-036).
fn resume_command_without_timeout(args: impl Iterator<Item = String>) -> String {
    resume_command_for(args, std::env::var_os("SPECSYNC_TIMEOUT").is_some())
}

/// `resume_command_without_timeout` with whether `SPECSYNC_TIMEOUT` is set passed in, so tests
/// do not depend on the environment they run in.
fn resume_command_for(args: impl Iterator<Item = String>, timeout_env: bool) -> String {
    let mut kept = Vec::new();
    if timeout_env {
        kept.push("SPECSYNC_TIMEOUT=".to_string());
    }
    kept.push("specsync".to_string());
    let mut args = args.peekable();
    while let Some(argument) = args.next() {
        if argument == "--timeout" {
            args.next();
            continue;
        }
        if argument.starts_with("--timeout=") {
            continue;
        }
        let shell_safe = !argument.is_empty()
            && argument
                .bytes()
                .all(|byte| byte.is_ascii_alphanumeric() || b"_@%+=:,./-".contains(&byte));
        kept.push(if shell_safe {
            argument
        } else {
            format!("'{}'", argument.replace('\'', "'\\''"))
        });
    }
    kept.join(" ")
}
fn print_record(
    root: &Path,
    record: &ChangeRecord,
    format: OutputFormat,
    include_questions: bool,
    strict: bool,
) -> Result<(), String> {
    let retirement = if record.state == ChangeState::Retired {
        change::validate_retirement(root, record)?;
        Some(change::retirement_record(root, record)?)
    } else {
        None
    };
    let questions = if include_questions {
        change::next_questions_for(root, record)
    } else {
        Vec::new()
    };
    // Who each recorded approval and review names, with any relayed decision (#633). Read first,
    // so a relay record that cannot be read refuses the command and names its recovery.
    let claims = change::recorded_claims(root, record)?;
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
            // Inside the status/show read scope, so this shares the delta-binding read `summary`
            // used a few lines above (REQ-cmd-change-023).
            let approved_deltas = change::approved_delta_summaries(root, record);
            print_json(
                &serde_json::json!({
                    "change": record,
                    "effective_definition": effective_definition,
                    "acceptance_entries": acceptance_entries,
                    "approved_deltas": approved_deltas,
                    "corrections": corrections,
                    "summary": summary,
                    "questions": questions,
                    "briefing": change::open_change_briefing(root, record),
                })
                .as_object()
                .map(|object| {
                    let mut object = object.clone();
                    if record
                        .answers
                        .get("historical_reconciliation")
                        .is_some_and(|value| value == "unproven")
                    {
                        object.insert(
                            "historical_transition_proven".into(),
                            serde_json::json!(false),
                        );
                    }
                    if let Some(retirement) = &retirement {
                        object.insert("retirement".into(), serde_json::json!(retirement));
                    }
                    if !claims.is_empty() {
                        object.insert("recorded_claims".into(), serde_json::json!(claims));
                    }
                    object
                })
                .unwrap_or_default(),
            );
        }
        _ => {
            // Text mode must not invoke digest-bearing loaders into cleartext sinks
            // (CodeQL rust/cleartext-logging / alert #58). Human output uses interview
            // identity/state only; digests and correction-ledger details stay JSON-only.
            ensure_text_correction_ledger_valid(root, record)?;
            // `text_mode_next_action` uses lightweight artifact file reads only.
            print_change_text_identity(record);
            print_open_change_briefing(root, record);
            let next = text_mode_next_action(root, record);
            println!("  Next: {next}");
            // The handoff line is literal prose plus the change ID; no digest reaches it.
            print_handoff_line(&change::handoff_summary(root, record));
            if let Some(retirement) = &retirement {
                println!(
                    "Retired without implementation acceptance; historical transitions unproven."
                );
                println!("  Recorded actor/source are claims, not authenticated human identity.");
                println!("  Replacement: {}", retirement.replacement_id);
                // Provenance the disposition was recorded with; text stays free of
                // digests (CodeQL rust/cleartext-logging), so only the sanitized
                // claim, the source kind, and per-input custody classes print.
                println!(
                    "  Authority: {}",
                    printable_claim_text(&retirement.authority_source)
                );
                if let Some(kind) = &retirement.source_kind {
                    println!("  Source: {}", retirement_source_kind_label(kind));
                }
                for entry in &retirement.custody {
                    if let Some(classification) = &entry.classification {
                        println!(
                            "  Custody: {} ({})",
                            printable_claim_text(&entry.path),
                            retirement_custody_classification_label(classification)
                        );
                    }
                }
            }
            print_recorded_claims(&claims);
            print_verification_ignored_lines(root);
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

/// Each recorded approval and review as the claim it is, with any relayed decision on the next
/// line (CHANGE-6.d, CHANGE-13.b, CHANGE-13.c). `RecordedClaim` holds names and relayed words
/// only, never a digest.
fn print_recorded_claims(claims: &[change::RecordedClaim]) {
    for claim in claims {
        let name = printable_claim_text(&claim.name);
        match (claim.gate.as_str(), claim.verdict.as_deref()) {
            ("review", Some(verdict)) => println!(
                "  Review: {name} ({}; {})",
                printable_claim_text(verdict),
                change::RECORDED_CLAIM_BOUNDARY
            ),
            _ => println!("  Approval: {name} ({})", change::RECORDED_CLAIM_BOUNDARY),
        }
        if let Some(relay) = &claim.relayed_by {
            println!(
                "    Relayed: {} said \"{}\" ({}); recorded by {}; {}",
                printable_claim_text(&relay.decided_by),
                printable_claim_text(&relay.words),
                printable_claim_text(&relay.where_),
                printable_claim_text(&relay.recorded_by),
                change::RELAYED_CLAIM_BOUNDARY
            );
        }
    }
}

/// One line per repaired relayed decision (REQ-cmd-change-relay-repair-001): where the original
/// was put back from, or that it was removed and what it held, escaped like every claim.
fn print_relay_repairs(report: &change::RelayRepairReport) {
    if report.repairs.is_empty() {
        println!("No relayed decision in {} needs repair.", report.change);
        return;
    }
    println!(
        "{} Repaired relayed decisions in {}:",
        "✓".green(),
        report.change
    );
    for repair in &report.repairs {
        let place = match repair.file.as_str() {
            "approvals.json" => format!("approvals.json approval {}", repair.record),
            "review-attempts.json" => format!("review-attempts.json attempt {}", repair.record),
            other => other.to_string(),
        };
        match &repair.from {
            Some(from) => println!("  {place}: put back from {from}"),
            None => println!(
                "  {place}: removed, because no copy of the original exists; it was {}",
                printable_claim_text(
                    &repair
                        .removed
                        .as_ref()
                        .map(ToString::to_string)
                        .unwrap_or_default()
                )
            ),
        }
    }
    println!(
        "  Every approval and review attempt is kept, verdicts included; no digest or gate reads a \
         relayed decision, so nothing went stale."
    );
    if report.repairs.iter().any(|repair| repair.from.is_none()) {
        println!(
            "  To carry a removed relayed decision again, record that decision again with \
             --words, --where and --recorded-by."
        );
    }
}

/// Recorded words are printed, not obeyed: a control character is shown as its escape, so a
/// relayed `words` field cannot move the cursor or recolour the terminal.
fn printable_claim_text(text: &str) -> String {
    let mut printable = String::with_capacity(text.len());
    for character in text.chars() {
        if character.is_control() {
            printable.extend(character.escape_debug());
        } else {
            printable.push(character);
        }
    }
    printable
}

/// Human label for a retirement receipt's recorded source kind (text status only).
fn retirement_source_kind_label(kind: &change::RetirementSourceKind) -> &'static str {
    match kind {
        change::RetirementSourceKind::AcceptedReceipt => "accepted receipt",
        change::RetirementSourceKind::CommittedAuditedReopeningUnproven => {
            "committed audited reopening (unproven)"
        }
    }
}

/// Human label for one custody entry's classification (text status only).
fn retirement_custody_classification_label(
    classification: &change::RetirementCustodyClassification,
) -> &'static str {
    match classification {
        change::RetirementCustodyClassification::SameSignedOwners => "same signed owners",
        change::RetirementCustodyClassification::UnchangedExactFileCurrentOwner => {
            "unchanged exact file under a current owner"
        }
    }
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
        change::next_questions_for(root, record)
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
            print_json(
                &serde_json::json!({
                    "change": record,
                    "effective_definition": result.effective_definition,
                    "acceptance_entries": acceptance_entries,
                    "approved_deltas": result.approved_deltas,
                    "corrections": result.corrections,
                    "summary": summary,
                    "questions": questions,
                    "briefing": change::open_change_briefing(root, record),
                })
                .as_object()
                .map(|object| {
                    let mut object = object.clone();
                    if record
                        .answers
                        .get("historical_reconciliation")
                        .is_some_and(|value| value == "unproven")
                    {
                        object.insert(
                            "historical_transition_proven".into(),
                            serde_json::json!(false),
                        );
                    }
                    object
                })
                .unwrap_or_default(),
            );
        }
        _ => {
            // The domain operation validated correction history while holding the mutation lock.
            // Do not turn successful persistence into a false command failure by rereading that
            // ledger after the transaction has completed.
            print_change_text_identity(record);
            print_open_change_briefing(root, record);
            let next = text_mode_next_action(root, record);
            println!("  Next: {next}");
            print_handoff_line(&change::handoff_summary(root, record));
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

/// The standing limit printed beside a non-empty `verification_ignored_paths` list.
const VERIFICATION_IGNORED_NOTE: &str = "SpecSync cannot tell whether project tests or CI read files under these prefixes; edits there do not stale verification.";

/// Text lines disclosing a non-empty `verification_ignored_paths` list; none when it is absent,
/// empty or invalid (an invalid list is the next action instead). REQ-cmd-change-024.
fn verification_ignored_lines(entries: &[String]) -> Vec<String> {
    if entries.is_empty() {
        return Vec::new();
    }
    vec![
        format!("  Verification ignores: {}", entries.join(", ")),
        format!("    note: {VERIFICATION_IGNORED_NOTE}"),
    ]
}

fn print_verification_ignored_lines(root: &Path) {
    for line in verification_ignored_lines(&change::verification_ignored_paths_for_display(root)) {
        println!("{line}");
    }
}

/// Print only non-sensitive identity fields for human text sinks.
///
/// Digests / correction ledgers must not flow into `println!` (CodeQL
/// `rust/cleartext-logging`). Callers must not pass values produced by
/// `effective_change_definition` / `correction_history` into this path.
fn print_change_text_identity(record: &ChangeRecord) {
    if record
        .answers
        .get("historical_reconciliation")
        .is_some_and(|value| value == "unproven")
    {
        println!(
            "  Historical file transitions remain unproven (explicit receipt reconciliation)."
        );
    }
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
        .filter(|(field, _)| field.as_str() != "historical_reconciliation_anchor")
        .map(|(field, value)| format!("{field}={value}"))
        .collect::<Vec<_>>()
        .join(", ");
    println!("  Answers: {answer_summary}");
}

/// Human next-step string from interview/state, lightweight artifact completeness, and approval
/// currency. Nothing digest-bearing reaches the string (CodeQL cleartext-logging): artifacts are
/// read through `artifacts_complete_for_guidance`, and approval currency arrives as the boolean
/// `approval_current_for_guidance` returns.
fn text_mode_next_action(root: &Path, record: &ChangeRecord) -> String {
    let id = record.id.as_str();
    // The same refusal every later verb would give, named before any state-local step
    // (REQ-change-115). It carries entry paths and rule text only, never a digest.
    if record.state != ChangeState::Archived
        && let Some(error) = change::verification_ignored_paths_refusal(root, Some(record))
    {
        return format!(
            "fix `.specsync/sdd.json` verification_ignored_paths ({error}), then run `specsync change status {id}`"
        );
    }
    // A project-wide sequence-ledger freeze blocks every verb, so it outranks the upstream
    // archive, in the same order as JSON status (#908, REQ-cmd-change-034).
    if let Some(remediation) = change::sequence_ledger_freeze_next_action(root) {
        return remediation;
    }
    // After the project-wide refusals above, in the same order as JSON status: an archive of
    // this id on the remote default makes review and finalize refuse (REQ-cmd-change-029).
    if let Some(upstream) = change::upstream_archive_of(root, record) {
        return change::upstream_archive_message(id, &upstream);
    }
    match record.state {
        ChangeState::Retired => change::retirement_next_action(root, record),
        // The command approve's refusal names for the same question
        // (REQ-cmd-change-answer-hint-001).
        ChangeState::Draft if !change::next_questions_for(root, record).is_empty() => {
            match change::next_questions_for(root, record).first() {
                Some(question) => format!(
                    "run {}",
                    change::answer_command_hint(id, &question.id, &question.choices)
                ),
                None => format!("run `specsync change answer {id} <question> \"<answer>\"`"),
            }
        }
        ChangeState::Draft if !change::artifacts_complete_for_guidance(root, record) => {
            format!("complete selected artifacts, then run `specsync change status {id}`")
        }
        ChangeState::Draft => {
            format!("run `specsync change approve {id} --actor <name>`")
        }
        // The same approval currency `summary.approval_valid` reports: a changed scope or an
        // edited approved delta. `check` would refuse both, so naming it here sent the operator
        // to a certain failure.
        ChangeState::Approved | ChangeState::Implementing | ChangeState::Verifying
            if !change::approval_current_for_guidance(root, record) =>
        {
            format!("run `specsync change approve {id} --actor <name>`")
        }
        ChangeState::Approved | ChangeState::Implementing => open_tasks_first(
            root,
            record,
            format!("run `specsync change check {id} --commit`"),
        ),
        ChangeState::Verifying => open_tasks_first(
            root,
            record,
            format!(
                "run `specsync change check {id} --commit` if needed, then `specsync change ship-status {id}` (or `ship {id}`) — scoped review then finalize before merging; merging first orphans verification evidence and blocks earlier accepted changes sharing a delivery input from archiving"
            ),
        ),
        ChangeState::Accepted if record.workflow_version >= 2 => open_tasks_first(
            root,
            record,
            format!("run `specsync change ship {id}` or `specsync change finalize {id}`"),
        ),
        ChangeState::Accepted => format!("run `specsync change archive {id}`"),
        // An archive that fails its integrity check is not finished: name the same restore the
        // JSON next action and the handoff carry (#880). Path and literals only, no digest.
        ChangeState::Archived => change::corrupt_archive_recovery_for_guidance(root, record)
            .unwrap_or_else(|| "no further action".into()),
    }
}

/// `action`, preceded by the change's open tasks when it has any. Checkbox state is outside every
/// digest, so no currency check above can see them, and every verb `action` names would refuse
/// (REQ-change-132).
fn open_tasks_first(root: &Path, record: &ChangeRecord, action: String) -> String {
    match change::open_tasks_next_step(root, record) {
        Some(step) => format!("{step}, then {action}"),
        None => action,
    }
}

fn ensure_text_correction_ledger_valid(root: &Path, record: &ChangeRecord) -> Result<(), String> {
    change::effective_change_definition(root, record)
        .is_ok()
        .then_some(())
        .ok_or_else(|| INVALID_CORRECTION_LEDGER_TEXT.to_string())
}

/// Ship readiness for one change: HEAD tip class, verification tip health, review,
/// archive-first CI guidance (verify and review locally, ship, push the archive tip), and the
/// merge-before-finalize trap.
///
/// When `GITHUB_TOKEN` is set and the git remote is a GitHub repo, queries check-runs for the
/// HEAD (or parent) SHA: archive-first CI runs the full lane only on a tip whose changes are
/// all archived, so HEAD is where the merge decision is made. Offline / no-token stays on `local_guidance`. Force offline with
/// `SPECSYNC_SHIP_LOCAL_GUIDANCE=1`.
fn ship_status_report(root: &Path, record: &ChangeRecord) -> Result<serde_json::Value, String> {
    let lifecycle_next = text_mode_next_action(root, record);
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
                // Only a full commit id, as SpecSync records it, reaches git.
                Some(commit) if crate::change::is_canonical_commit_id(&commit) => {
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
    // Currency below is read through boolean helpers that turn the digest's refusal of an invalid
    // `verification_ignored_paths` list into "stale". Name the entry instead (REQ-cmd-change-024).
    let verification_ignored_refusal = (record.state != ChangeState::Archived)
        .then(|| change::verification_ignored_paths_refusal(root, Some(record)))
        .flatten();
    if let Some(refusal) = &verification_ignored_refusal {
        blockers.push(format!(
            "{refusal}; fix `.specsync/sdd.json` before check, review or finalize"
        ));
    }
    // A project-wide sequence-ledger freeze outranks the upstream archive: rebasing does not
    // clear it, so it is the first blocker and the next step, as in `change status` (#908).
    let sequence_freeze = change::sequence_ledger_freeze_next_action(root);
    if let Some(remediation) = &sequence_freeze {
        blockers.push(remediation.clone());
    }
    let upstream_archive = change::upstream_archive_of(root, record);
    if let Some(upstream) = &upstream_archive {
        blockers.push(change::upstream_archive_message(&record.id, upstream));
    }
    // No digest sees checkbox state, so the currency blockers below cannot report an open task;
    // check, review, finalize and ship would each refuse it (REQ-cmd-change-031). A change already
    // archived upstream is told that instead.
    let open_tasks_step = (upstream_archive.is_none() && change::open_tasks_block(record))
        .then(|| change::open_tasks_next_step(root, record))
        .flatten();
    if let Some(step) = &open_tasks_step {
        blockers.push(format!("tasks.md contains incomplete tasks: {step}"));
    }
    // Finalize refuses production source no declared module owns; ship-status and ship
    // preflight run the same check so "ready to finalize" never names a change it refuses
    // (REQ-cmd-change-008, #986, #956).
    let ownership_recovery = if record.state == ChangeState::Verifying && upstream_archive.is_none()
    {
        match change::ownership_preflight_blocker(root, record) {
            Ok(Some((blocker, recovery))) => {
                blockers.push(blocker);
                Some(recovery)
            }
            Ok(None) => None,
            Err(error) => {
                blockers.push(format!(
                    "the acceptance ownership preflight could not run ({error}); finalize would not either"
                ));
                None
            }
        }
    } else {
        None
    };
    let verification_work = if verification_commit.is_some() && verification_present {
        change::recorded_verification_commit_contains_change_work(root, record)
    } else {
        Ok(())
    };
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
        } else if verification_ignored_refusal.is_none()
            && !change::recorded_verification_is_current(root, record)
        {
            // #689: the question is whether the CONTENT still matches, not whether the recorded
            // commit is still reachable. A squash-merge guarantees it is not — and squash is the
            // only strategy this repository, and most repositories, permit. Blocking on
            // reachability made a squash-merged change permanently unfinalizable while its
            // evidence was perfectly good.
            blockers.push(
                "verification evidence is stale for the current tree; re-run change check --commit before review/finalize"
                    .to_string(),
            );
        } else if let Err(error) = &verification_work {
            blockers.push(error.clone());
        }
        if record.state == ChangeState::Verifying {
            // Three outcomes, three sentences. A DECIDED negative is a blocker, matching the
            // verification half directly above; an UNDECIDED one is a warning, because saying
            // "you are blocked" would settle #694's open question by accident. What neither
            // of them may do is stay silent and let readiness read the silence as a pass.
            match &review_currency {
                None if !review_present => warnings.push(
                    "no scoped review recorded yet; finalize requires scoped review"
                        .to_string(),
                ),
                None => blockers.push(format!(
                    "the recorded scoped review cannot be read; re-run `specsync change review {} --reviewer <human>` before finalize",
                    record.id
                )),
                Some(change::ScopedReviewCurrency::Current) => {}
                Some(change::ScopedReviewCurrency::Stale(reason)) => blockers.push(format!(
                    "scoped review is stale — {reason}; re-run `specsync change review {} --reviewer <human>` before finalize",
                    record.id
                )),
                Some(change::ScopedReviewCurrency::Unavailable(reason)) => warnings.push(format!(
                    "scoped review currency could not be determined — {reason}; readiness cannot confirm it and finalize may refuse. Re-recording the review with `specsync change review {} --reviewer <human>` re-anchors it",
                    record.id
                )),
            }
            warnings.push(merge_before_finalize_warning(false));
        }
        if tip.tip_class == "archive_only"
            && !matches!(record.state, ChangeState::Archived | ChangeState::Retired)
        {
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
    // Retired is terminal, so these gates no longer apply to it either.
    if !matches!(record.state, ChangeState::Archived | ChangeState::Retired) {
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
    let verification_current =
        change::recorded_verification_is_current(root, record) && verification_work.is_ok();
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
        verification_current,
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

    let ship_next = if let Some(remediation) = &sequence_freeze {
        remediation.clone()
    } else if let Some(upstream) = &upstream_archive {
        change::upstream_archive_message(&record.id, upstream)
    } else if let Some(recovery) = &ownership_recovery {
        recovery.clone()
    } else if verification_work.is_err() {
        format!("run `specsync change check {} --commit`", record.id)
    } else if ready_to_finalize {
        format!("run `specsync change ship {}`", record.id)
    } else if matches!(
        record.state,
        ChangeState::Draft | ChangeState::Accepted | ChangeState::Archived | ChangeState::Retired
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
        let stage_action = stages
            .iter()
            .find(|stage| stage.get("status").and_then(|value| value.as_str()) == Some("current"))
            .and_then(|stage| stage.get("action"))
            .and_then(|value| value.as_str());
        match (stage_action, &open_tasks_step) {
            // A stage's action is about evidence; the open tasks come first (REQ-cmd-change-031).
            (Some(action), Some(step)) => format!("{step}, then {action}"),
            (Some(action), None) => action.to_string(),
            // `lifecycle_next` already names any open tasks.
            (None, _) => lifecycle_next.clone(),
        }
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
        .filter(|record| {
            record.id != id && !matches!(record.state, ChangeState::Archived | ChangeState::Retired)
        })
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

const SHIP_TRUST_LOCAL_GUIDANCE: &str = "Archive-first CI: a pushed tip with any open change fails the Archive job and runs nothing else, so do not wait on CI for an unarchived tip. Verify locally (`specsync change check <id> --commit` and the local verify lane), record `change review`, run `change ship` without committing in between, then commit and push the archive tip. CI runs the full lane on that tip; merge when it is green.";

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
    // Prefer HEAD: archive-first CI runs the full lane on the archived tip itself, and no
    // tip reuses trust from a parent any more. The parent is only a fallback.
    let query_sha = tip
        .tip_sha
        .as_deref()
        .or(tip.parent_sha.as_deref())
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
                    "GitHub check-runs on {query_sha:.8} are green. If a change is still open, ship it and push the archive tip: archive-first CI decides the merge on that tip."
                ),
                "pending" => format!(
                    "GitHub check-runs on {query_sha:.8} are still pending — wait for them to finish. Archive-first CI runs the full lane only on a tip whose changes are all archived."
                ),
                "failed" => format!(
                    "GitHub check-runs on {query_sha:.8} failed or cancelled. A failed Archive on a tip with an open change is expected: ship locally, then push the archive tip. Any other failure on an archived tip needs a fix commit on top, with specs kept in sync (`specsync check --strict`)."
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

/// Show the same briefing text and JSON carry. A bounded excerpt, then the path for the rest.
fn print_open_change_briefing(root: &Path, record: &change::ChangeRecord) {
    let briefing = change::open_change_briefing(root, record);
    if briefing.lessons.is_empty() && briefing.principles.is_none() {
        return;
    }
    if !briefing.lessons.is_empty() {
        println!(
            "\n  {} what these modules already learned:",
            "Lessons:".bold()
        );
        for lesson in &briefing.lessons {
            let more = if lesson.truncated {
                ", more in the file"
            } else {
                ""
            };
            println!("    {} ({} line(s){more})", lesson.path, lesson.lines);
            for line in lesson.excerpt.lines() {
                println!("      {line}");
            }
        }
    }
    if let Some(principles) = &briefing.principles {
        println!("\n  {} {}", "Principles:".bold(), principles.path);
        if principles.missing {
            println!("    the file could not be read");
        } else {
            for line in principles.excerpt.lines() {
                println!("    {line}");
            }
            if principles.truncated {
                println!("    more in the file");
            }
        }
    }
}

fn lesson_bundle_line() -> String {
    // The file name only. The archive directory is printed on its own line from
    // `finalize_change`'s path, and formatting that path into this string is what
    // CodeQL `rust/cleartext-logging` treats as logging correction history.
    format!("Lesson bundle: {}", change::LESSON_BUNDLE_FILE)
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
/// One open change, in the order the pull request names them.
#[derive(Clone)]
struct ArchiveCandidate {
    id: String,
    state: String,
    /// Verifying, with review and verification files on disk. The ship command is the next step.
    ship: bool,
}

fn ordered_archive_id(open: &[ArchiveCandidate]) -> Option<String> {
    let mut ordered = open.to_vec();
    ordered.sort_by(|left, right| {
        archive_rank(&left.state)
            .cmp(&archive_rank(&right.state))
            .then(left.id.cmp(&right.id))
    });
    ordered.into_iter().next().map(|candidate| candidate.id)
}

fn archive_rank(state: &str) -> u8 {
    match state {
        // Accepted is closest to the archive: it resumes at finalize, not at a new check.
        "accepted" => 0,
        "verifying" => 1,
        "approved" => 2,
        "implementing" => 3,
        "draft" => 4,
        _ => 5,
    }
}

/// The Archive sentence without its `Run ...` tail.
///
/// `change status` already prints the real next action underneath. Repeating
/// `Run specsync change status <id>` there tells the operator to run the command
/// they are already in.
fn archive_sentence_without_command(sentence: &str) -> String {
    match sentence.rfind(" Run `") {
        Some(index) => {
            let mut text = sentence[..index].trim_end().to_string();
            if !text.ends_with('.') {
                text.push('.');
            }
            text
        }
        None => sentence.to_string(),
    }
}

/// The sentence the Archive check prints. Empty means every change is archived.
///
/// The same words are in `.github/scripts/archive-readiness.sh`.
fn pull_request_archive_sentence(open: &[ArchiveCandidate]) -> String {
    if open.is_empty() {
        return "All changes are archived. Ready to merge when the other checks are green."
            .to_string();
    }
    let mut ordered = open.to_vec();
    ordered.sort_by(|left, right| {
        archive_rank(&left.state)
            .cmp(&archive_rank(&right.state))
            .then(left.id.cmp(&right.id))
    });
    let first = &ordered[0];
    let command = if first.ship {
        format!("specsync change ship {}", first.id)
    } else {
        format!("specsync change status {}", first.id)
    };
    if ordered.len() == 1 {
        format!(
            "This pull request is not finished. `{}` is still open. Run `{command}`.",
            first.id
        )
    } else {
        let still_open = ordered
            .iter()
            .map(|candidate| format!("`{}`", candidate.id))
            .collect::<Vec<_>>()
            .join(", ");
        format!(
            "This pull request is not finished. Archive `{}` first. Still open: {still_open}. Run `{command}`.",
            first.id
        )
    }
}

fn open_archive_candidates(root: &Path) -> Result<Vec<ArchiveCandidate>, String> {
    let roster = change::list_changes(root)?;
    let mut open = Vec::new();
    for record in roster.records {
        // `list_changes` only reads `.specsync/changes/`. A record whose state
        // says archived but that still sits in the active tree is not finished.
        // Skipping it told `change status` to merge while the Archive check failed.
        let dir = change::find_change_dir(root, &record.id)?;
        let state = if record.state == change::ChangeState::Archived {
            "archived".to_string()
        } else {
            record.state.as_str().to_string()
        };
        let ship = archive_candidate_can_ship(root, &record, &dir);
        open.push(ArchiveCandidate {
            id: record.id,
            state,
            ship,
        });
    }
    for unreadable in roster.unreadable {
        open.push(ArchiveCandidate {
            id: unreadable.id,
            state: "unreadable".to_string(),
            ship: false,
        });
    }
    Ok(open)
}

/// Whether the Archive check may tell this workspace to run `change ship`.
///
/// A blocking review, a legacy workflow, a review recorded against a different
/// commit, or a copy the remote default already archived cannot be archived by
/// that command. The same proof is in `.github/scripts/archive-readiness.sh`:
/// verdict `pass`, workflow v2, `implementation_commit` equal to HEAD, and no
/// upstream archive. Anything the check cannot prove stays on `change status`.
fn archive_candidate_can_ship(root: &Path, record: &ChangeRecord, dir: &Path) -> bool {
    if record.workflow_version < 2 || record.state != change::ChangeState::Verifying {
        return false;
    }
    if !dir.join("verification.json").is_file() {
        return false;
    }
    let Ok(text) = fs::read_to_string(dir.join("review.json")) else {
        return false;
    };
    let Ok(review) = serde_json::from_str::<serde_json::Value>(&text) else {
        return false;
    };
    if review.get("verdict").and_then(|value| value.as_str()) != Some("pass") {
        return false;
    }
    let Some(commit) = review
        .get("implementation_commit")
        .and_then(|value| value.as_str())
    else {
        return false;
    };
    let reviewed_at_head = git_rev_parse(root, "HEAD")
        .ok()
        .is_some_and(|head| head == commit);
    // Asked last, so Git runs only for a copy that would otherwise ship. `ship` refuses a
    // change the remote default already archived (REQ-change-129); naming it here sent the
    // operator to a certain refusal instead of to `change status`, which names the recovery.
    // Last of all, and only for a change that would otherwise ship: `ship` refuses production
    // source no declared module owns (REQ-cmd-change-008), so such a change is sent to
    // `change status`, which names the recovery (#986, #956).
    reviewed_at_head
        && change::upstream_archive_of(root, record).is_none()
        && matches!(change::ownership_preflight_blocker(root, record), Ok(None))
}

fn pull_request_sentence(root: &Path) -> Result<String, String> {
    Ok(pull_request_archive_sentence(&open_archive_candidates(
        root,
    )?))
}

fn ship_next_action(
    push: bool,
    wait: bool,
    siblings_before: &[String],
    fold_targets: &[String],
    bundle: &str,
) -> String {
    let _ = (fold_targets, bundle);
    if let Some(next_id) = siblings_before.first() {
        return format!("Archive `{next_id}` next. This pull request is not finished.");
    }
    if push && wait {
        "Merge the pull request when the checks are green.".to_string()
    } else if push {
        "Wait until the checks are green, then merge the pull request.".to_string()
    } else {
        "Push this archive, wait until the checks are green, then merge the pull request."
            .to_string()
    }
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
    "Push this archive, wait until the checks are green, then merge the pull request.".to_string()
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

/// The product-tip action, named only as a command whose state guard admits the change's state
/// (#690, REQ-cmd-change-033). `check` refuses a draft and an accepted or archived change, so
/// those get the step their own state machine takes: approve, or reopen the stale evidence.
fn product_stage_action(record: &ChangeRecord) -> String {
    let id = record.id.as_str();
    match record.state {
        ChangeState::Retired => {
            "retired without implementation acceptance; use the independent current replacement"
                .into()
        }
        ChangeState::Draft => format!(
            "run `specsync change approve {id} --actor <name>` once the definition is complete; `change check` refuses a draft"
        ),
        ChangeState::Accepted | ChangeState::Archived => format!(
            "run `specsync change reopen {id} --actor <name> --reason <text>`; the verification is no longer current for this content and `change check` refuses an accepted or archived change"
        ),
        ChangeState::Approved | ChangeState::Implementing | ChangeState::Verifying => format!(
            "run `specsync change check {id} --commit` and the local verify lane; archive-first CI does not test a tip with an open change, so do not wait on remote CI here"
        ),
    }
}

fn ship_stages(
    record: &ChangeRecord,
    tip: &HeadTip,
    verification_commit: Option<&str>,
    verification_present: bool,
    verification_current: bool,
    review: ShipReviewStatus,
    ready_to_finalize: bool,
) -> Vec<serde_json::Value> {
    let id = record.id.as_str();
    let verified = verification_present && verification_current && verification_commit.is_some();
    let mut stages = Vec::new();

    // Retired is a completed terminal disposition in every stage: no product tip,
    // review, finalize, or merge gate applies to a record that was retired
    // without implementation acceptance.
    let retired = record.state == ChangeState::Retired;
    let product_done = retired
        || (verified
            && matches!(
                record.state,
                ChangeState::Verifying | ChangeState::Accepted | ChangeState::Archived
            ));
    stages.push(serde_json::json!({
        "id": "product_tip",
        "title": "Product tip (local verification)",
        "status": if product_done { "done" } else { "current" },
        "sha": verification_commit,
        "action": if retired {
            product_stage_action(record)
        } else if product_done {
            "product verification evidence is current for this content".to_string()
        } else {
            product_stage_action(record)
        },
    }));

    // The stage tracks currency, not existence (#743). A `done` review tip beside a
    // `ready_to_finalize: false` is the same recommend-then-refuse contradiction one line
    // lower, and it is also what leaves `current_stage` empty — which drops `ship_next` back
    // to the generic lifecycle line instead of naming the recovery.
    let review_status = if retired || review.current() {
        "done"
    } else if product_done {
        "current"
    } else {
        "pending"
    };
    stages.push(serde_json::json!({
        "id": "review_tip",
        "title": "Scoped review (local, no CI wait)",
        "status": review_status,
        "action": if retired {
            "no scoped review applies; the change was retired without implementation acceptance".to_string()
        } else if review.current() {
            "scoped review is recorded and current".to_string()
        } else if product_done && review.present() {
            format!(
                "re-record the scoped review: `specsync change review {id} --reviewer <human>` — the recorded review is `{}` against this tree, and finalize requires a current one",
                review.label()
            )
        } else if product_done {
            format!(
                "record scoped review: `specsync change review {id} --reviewer <human>`, then run `specsync change ship {id}` without committing in between"
            )
        } else {
            "complete product tip first".to_string()
        },
    }));

    let archive_status = if record.state == ChangeState::Archived || retired {
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
        "action": if retired {
            "retirement custody is recorded as terminal nonacceptance; no finalize applies".to_string()
        } else if record.state == ChangeState::Archived {
            "change is archived; commit and push the archive tip, wait for CI (Archive, then the full lane), then merge when it is green"
                .to_string()
        } else if ready_to_finalize {
            format!("run `specsync change ship {id}`")
        } else {
            "complete product tip and scoped review first".to_string()
        },
    }));

    stages.push(serde_json::json!({
        "id": "merge",
        "title": "Merge the PR",
        "status": if retired {
            "done"
        } else if record.state == ChangeState::Archived {
            "current"
        } else {
            "pending"
        },
        "action": if retired {
            "retired without implementation acceptance; this record has no finalize or merge gate".to_string()
        } else {
            "merge only after finalize — merging an unfinalized change orphans verification evidence".to_string()
        },
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

/// Archive the change when it is still open, push the branch, and open or reuse the pull request.
///
/// Merging stays a separate act. A change that is not ready is refused with the one command
/// that would make it ready.
fn run_prepare_pr(root: &Path, id: Option<&str>, format: OutputFormat) -> Result<(), String> {
    let record = resolve_ship_record(root, id)?;
    let mut archived_now = false;
    if record.state != ChangeState::Archived {
        let report = ship_status_report(root, &record)?;
        let ready = report["ready_to_finalize"].as_bool() == Some(true);
        let open = open_archive_candidates(root)?;
        let first = ordered_archive_id(&open);
        let this_is_first = first.as_deref() == Some(record.id.as_str()) || open.is_empty();
        if ready && this_is_first {
            // Quiet: this command prints one document after the pull request exists.
            run_ship(
                root,
                Some(&record.id),
                false,
                false,
                false,
                900,
                OutputFormat::Text,
                false,
            )?;
            archived_now = true;
        }
    } else {
        // Re-running ship on an archived change must not sweep unrelated edits
        // into that change's archive commit.
        let archive_commit = git_commit_lifecycle(
            root,
            &change::lifecycle_commit_scope(root, &record.id)?,
            &format!("chore(lifecycle): archive {}", record.id),
            false,
        )?;
        disclose_left_out(&archive_commit.left_out, &mut Vec::new(), None);
    }
    // The pull request has to exist before review. A change that is not ready
    // still gets its branch pushed and its pull request opened, then stops.
    run_pre_push_gate(root)?;
    push_current_branch(root)?;
    let url = ensure_pull_request(root, &record.title, &record.description)?;
    let sentence = pull_request_sentence(root)?;
    let next = if sentence.starts_with("All changes are archived") {
        pull_request_ready_next().to_string()
    } else {
        sentence
    };
    match format {
        OutputFormat::Json => print_json(&serde_json::json!({
            "id": record.id,
            "pull_request": url,
            "merged": false,
            "archived": archived_now || record.state == ChangeState::Archived,
            "next": next,
        })),
        _ => {
            println!("{} {url}", "Pull request:".bold());
            println!("  Next: {next}");
        }
    }
    Ok(())
}

fn run_pre_push_gate(root: &Path) -> Result<(), String> {
    let status = std::process::Command::new("fledge")
        .args(["lanes", "run", "pre-push"])
        .current_dir(root)
        .status();
    // The gate is an external process; nothing git answered before it is trusted after it.
    crate::git_utils::invalidate_repository_facts();
    let status = status.map_err(|error| {
            format!(
                "pre-push gate could not start (`fledge lanes run pre-push`): {error}. The branch was not pushed."
            )
        })?;
    if !status.success() {
        return Err("pre-push gate failed. The branch was not pushed.".into());
    }
    Ok(())
}

fn resolve_ship_record(root: &Path, id: Option<&str>) -> Result<ChangeRecord, String> {
    match id {
        Some(id) => change::load_change(root, id),
        None => {
            let roster = change::list_changes(root)?;
            if let Some(unreadable) = roster.unreadable.first() {
                return Err(format!(
                    "cannot infer which change to prepare while a workspace is unreadable: {}; pass an explicit change id",
                    unreadable.reason
                ));
            }
            match roster.records.as_slice() {
                [] => Err("no active change to prepare; pass an explicit change id".into()),
                [single] => Ok(single.clone()),
                _ => Err(
                    "multiple active changes; pass an explicit id: `specsync change pr <ID>`"
                        .into(),
                ),
            }
        }
    }
}

fn push_current_branch(root: &Path) -> Result<(), String> {
    let branch = git_output_text(root, &["rev-parse", "--abbrev-ref", "HEAD"])?;
    let branch = branch.trim();
    if branch.is_empty() || branch == "HEAD" {
        return Err("cannot push a pull request from a detached HEAD".into());
    }
    let upstream = git_output_text(
        root,
        &[
            "rev-parse",
            "--abbrev-ref",
            "--symbolic-full-name",
            "@{upstream}",
        ],
    );
    if upstream.is_ok() {
        run_git(root, &["push"])?;
    } else {
        run_git(root, &["push", "-u", "origin", branch])?;
    }
    Ok(())
}

fn ensure_pull_request(root: &Path, title: &str, body: &str) -> Result<String, String> {
    let existing = std::process::Command::new("gh")
        .args(["pr", "view", "--json", "url,state"])
        .current_dir(root)
        .output()
        .map_err(|error| format!("failed to run gh pr view: {error}"))?;
    if existing.status.success()
        && let Ok(value) = serde_json::from_slice::<serde_json::Value>(&existing.stdout)
        && value.get("state").and_then(|state| state.as_str()) == Some("OPEN")
        && let Some(url) = value.get("url").and_then(|url| url.as_str())
        && !url.is_empty()
    {
        return Ok(url.to_string());
    }
    let mut command = std::process::Command::new("gh");
    command
        .args(["pr", "create", "--title", title, "--body", body])
        .current_dir(root);
    // Omit `--base` when this checkout cannot see `origin/HEAD`. `gh pr create`
    // then uses the repository's default branch instead of a hard-coded `main`.
    if let Some(base) = discovered_pr_base(root) {
        command.args(["--base", &base]);
    }
    let created = command.output();
    // `gh pr create` may push the branch, which moves remote-tracking refs.
    crate::git_utils::invalidate_repository_facts();
    let created = created.map_err(|error| format!("failed to run gh pr create: {error}"))?;
    if !created.status.success() {
        return Err(format!(
            "gh pr create failed: {}",
            String::from_utf8_lossy(&created.stderr).trim()
        ));
    }
    let url = String::from_utf8_lossy(&created.stdout).trim().to_string();
    if url.is_empty() {
        return Err("gh pr create succeeded without printing a url".into());
    }
    Ok(url)
}

fn pull_request_ready_next() -> &'static str {
    "Wait until the checks are green, then merge the pull request."
}

fn discovered_pr_base(root: &Path) -> Option<String> {
    git_output_text(root, &["rev-parse", "--abbrev-ref", "origin/HEAD"])
        .ok()
        .and_then(|value| value.trim().strip_prefix("origin/").map(str::to_string))
        .filter(|base| !base.is_empty())
}

/// `change ship --wait-timeout-secs` when the flag is not given; the clap default in `cli.rs`,
/// pinned to it by `ship_wait_timeout_default_matches_the_cli`.
const SHIP_WAIT_TIMEOUT_DEFAULT_SECS: u64 = 900;

/// The `change ship` command line that repeats an invocation with these flags.
fn ship_command(id: &str, dry_run: bool, push: bool, wait: bool, wait_timeout_secs: u64) -> String {
    let mut command = format!("specsync change ship {id}");
    for (given, flag) in [
        (dry_run, " --dry-run"),
        (push, " --push"),
        (wait, " --wait"),
    ] {
        if given {
            command.push_str(flag);
        }
    }
    if wait && wait_timeout_secs != SHIP_WAIT_TIMEOUT_DEFAULT_SECS {
        command.push_str(&format!(" --wait-timeout-secs {wait_timeout_secs}"));
    }
    command
}

#[allow(clippy::too_many_arguments)]
fn run_ship(
    root: &Path,
    id: Option<&str>,
    dry_run: bool,
    push: bool,
    wait: bool,
    wait_timeout_secs: u64,
    format: OutputFormat,
    announce: bool,
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
        // Only the archive package. A later dirty edit is not this change.
        let archive_commit = git_commit_lifecycle(
            root,
            &change::lifecycle_commit_scope(root, &record.id)?,
            &format!("chore(lifecycle): archive {}", record.id),
            false,
        )?;
        disclose_left_out(&archive_commit.left_out, &mut Vec::new(), None);
        if push || wait {
            let push_result = if push {
                run_git(root, &["push"])?;
                let sha = git_rev_parse(root, "HEAD").unwrap_or_else(|_| "HEAD".into());
                Some(format!("pushed archive tip {sha:.8}"))
            } else {
                None
            };
            let wait_result = if wait && !push && archive_commit.committed {
                Some(local_archive_not_pushed())
            } else if wait {
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
                        println!("  Next: {}", pull_request_sentence(root)?);
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
                    "{} {} is already archived. {}",
                    "✓".green(),
                    record.id,
                    pull_request_sentence(root)?
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

    // Soft preflight: live trust is advisory. `--wait` waits on the pushed archive tip itself.
    if announce && matches!(trust_status, "pending" | "failed" | "empty") {
        match format {
            OutputFormat::Json => {}
            _ => {
                println!(
                    "{} HEAD check-runs are `{trust_status}` — expected under archive-first CI while a change is open; ship, then push the archive tip and wait for CI there (see ship-status trust)",
                    "⚠".yellow()
                );
            }
        }
    }

    let open = open_archive_candidates(root)?;
    let before = pull_request_archive_sentence(&open);
    let first_id = ordered_archive_id(&open);
    if first_id.as_deref() != Some(record.id.as_str()) && !open.is_empty() {
        // One document. Printing here and returning Err made JSON mode emit two.
        return Err(before);
    }
    // Before the generic readiness refusal, so the command to re-run is this one, flags and all
    // (REQ-cmd-change-031). A change already archived upstream gets that blocker below instead.
    if change::open_tasks_block(&record)
        && change::upstream_archive_of(root, &record).is_none()
        && let Some(refusal) = change::open_tasks_refusal(
            root,
            &record,
            &ship_command(&record.id, dry_run, push, wait, wait_timeout_secs),
        )
    {
        return Err(refusal);
    }
    if !ready {
        let action = report["ship_next"]
            .as_str()
            .unwrap_or("specsync change status");
        let reason = report["blockers"]
            .as_array()
            .and_then(|values| values.first())
            .and_then(|value| value.as_str())
            .unwrap_or("this change is not ready to archive");
        // `ship_next` is already the action. Wrapping it in another `Run \`...\``
        // nested the backticks and hid the command.
        return Err(format!(
            "This pull request is not finished. {reason} {action}"
        ));
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
    let archive_commit = git_commit_lifecycle(
        root,
        &change::lifecycle_commit_scope(root, &record.id)?,
        &format!("chore(lifecycle): archive {}", record.id),
        true,
    )?;
    disclose_left_out(&archive_commit.left_out, &mut Vec::new(), None);

    let mut push_result = None;
    let mut wait_result = None;
    if push {
        run_git(root, &["push"])?;
        let sha = git_rev_parse(root, "HEAD").unwrap_or_else(|_| "HEAD".into());
        push_result = Some(format!("pushed archive tip {sha:.8}"));
    }
    if wait && !push && archive_commit.committed {
        wait_result = Some(local_archive_not_pushed());
    } else if wait {
        wait_result = Some(wait_for_head_check_runs(root, wait_timeout_secs, format)?);
    }

    let after = pull_request_sentence(root)?;
    let next = if after.starts_with("All changes are archived") {
        ship_next_action(
            push,
            wait,
            &[],
            &change::lesson_fold_targets(root, &record.id),
            &path.join(change::LESSON_BUNDLE_FILE).display().to_string(),
        )
    } else {
        after
    };
    if !announce {
        return Ok(());
    }
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
            println!("  {}", lesson_bundle_line());
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
            print_handoff_line_for(root, &record.id);
        }
    }
    Ok(())
}

fn local_archive_not_pushed() -> serde_json::Value {
    serde_json::json!({
        "status": "not_pushed",
        "detail": "the archive commit is only local; rerun with --push to wait on its checks",
    })
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
    let output = crate::git_utils::output_after_child_exits(
        std::process::Command::new("git")
            .args(["cat-file", "-e", &format!("{commit}^{{commit}}")])
            .current_dir(root),
    )
    .map_err(|error| format!("git cat-file failed: {error}"))?;
    Ok(output.status.success())
}

fn git_is_ancestor(root: &Path, maybe_ancestor: &str, tip: &str) -> Result<bool, String> {
    let output = crate::git_utils::output_after_child_exits(
        std::process::Command::new("git")
            .args(["merge-base", "--is-ancestor", maybe_ancestor, tip])
            .current_dir(root),
    )
    .map_err(|error| format!("git merge-base failed: {error}"))?;
    Ok(output.status.success())
}

fn git_rev_parse(root: &Path, rev: &str) -> Result<String, String> {
    let output = crate::git_utils::output_after_child_exits(
        std::process::Command::new("git")
            .args(["rev-parse", "--verify", rev])
            .current_dir(root),
    )
    .map_err(|error| format!("git rev-parse failed: {error}"))?;
    if !output.status.success() {
        return Err(format!("git rev-parse {rev} failed"));
    }
    Ok(String::from_utf8_lossy(&output.stdout).trim().to_string())
}

fn git_diff_name_only(root: &Path, from: &str, to: &str) -> Result<Vec<String>, String> {
    let output = crate::git_utils::output_after_child_exits(
        std::process::Command::new("git")
            .args(["diff", "--name-only", from, to])
            .current_dir(root),
    )
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
    let output = crate::git_utils::output_after_child_exits(
        std::process::Command::new("git")
            .args(["show", "--pretty=format:", "--name-only", commit])
            .current_dir(root),
    )
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
    let output = crate::git_utils::output_after_child_exits(
        std::process::Command::new("git")
            .args(["status", "--porcelain", "-z"])
            .current_dir(root),
    )
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
            .map(|record| summary_with_briefing(root, record, strict))
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

fn summary_with_briefing(root: &Path, record: &ChangeRecord, strict: bool) -> serde_json::Value {
    let summary = change::summarize_change_with_strict(root, record, strict);
    let mut value = serde_json::to_value(summary).unwrap_or(serde_json::Value::Null);
    if let Some(object) = value.as_object_mut() {
        object.insert(
            "briefing".to_string(),
            serde_json::to_value(change::open_change_briefing(root, record))
                .unwrap_or(serde_json::Value::Null),
        );
    }
    value
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
                .map(|record| summary_with_briefing(root, record, strict))
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
                let id = record.id.clone();
                let title = record.title.clone();
                let state = record.state.as_str().to_owned();
                let next = text_mode_next_action(root, record);
                println!("{:<14}  {state:<13}  {title}  next: {next}", id.bold());
                print_open_change_briefing(root, record);
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

/// The approve transition, plus the handoff verdict the new state produces: text gets the
/// `Handoff:` line, JSON gets it as `handoff` beside the transition.
fn print_transition_with_handoff(
    root: &Path,
    record: &ChangeRecord,
    format: OutputFormat,
    message: &str,
) {
    let handoff = change::handoff_summary(root, record);
    match format {
        OutputFormat::Json => print_json(&serde_json::json!({
            "id": record.id,
            "state": record.state,
            "message": message,
            "handoff": handoff,
        })),
        _ => {
            print_transition(record, format, message);
            print_handoff_line(&handoff);
        }
    }
}

/// The `Handoff:` line every lifecycle text result ends with: whether clearing context now
/// loses anything, in the domain's words. `HandoffSummary` is built from literals and the
/// change ID only, so no digest can reach this sink through it.
fn print_handoff_line(handoff: &change::HandoffSummary) {
    let mut line = format!(
        "  Handoff: {} — {}",
        handoff.readiness.as_str(),
        handoff.reason
    );
    if !handoff.before_clearing.is_empty() {
        line.push_str(". Before clearing: ");
        line.push_str(&handoff.before_clearing.join("; "));
    }
    println!("{line}");
}

/// Reloads the record by ID — after review or finalize the caller holds only the evidence —
/// and prints its handoff line. A record that cannot be reloaded prints nothing rather than a
/// guess; the command's own result already stands.
fn print_handoff_line_for(root: &Path, id: &str) {
    if let Ok(record) = change::load_change(root, id) {
        print_handoff_line(&change::handoff_summary(root, &record));
    }
}

/// The report `change status` and `change ship-status` give for a closed-late change, or `None`
/// when `id` resolves to a live or archived workspace, which an older SpecSync can mint again after
/// a late close and which is the record every lifecycle verb acts on (Codex review of #994).
fn closed_late_report(
    root: &Path,
    id: &str,
    format: OutputFormat,
    phase: &str,
) -> Option<Result<(), String>> {
    if !change::is_only_closed_late(root, id) {
        return None;
    }
    // The read deadline covers this lookup like every other status read.
    if let Err(error) = change::check_read_deadline(phase, 0, 1) {
        return Some(Err(error));
    }
    print_closed_late(root, id, format)
}

/// Who closed a change late, when and why, the commit that merged it, and that it was never
/// re-checked and is not verified (REQ-change-143). `None` when `id` was not closed late.
fn print_closed_late(root: &Path, id: &str, format: OutputFormat) -> Option<Result<(), String>> {
    let (package, record) = change::closed_late_record(root, id)?;
    let sentence = change::closed_late_sentence(root, id)?;
    let package = package
        .strip_prefix(root)
        .unwrap_or(&package)
        .to_string_lossy()
        .replace('\\', "/");
    match format {
        OutputFormat::Json => {
            let closed_late = match &record {
                Ok(record) => serde_json::to_value(record).unwrap_or(serde_json::Value::Null),
                Err(error) => serde_json::json!({ "error": error, "verified": false }),
            };
            print_json(&serde_json::json!({
                "id": id,
                "closed_late": closed_late,
                "package": package,
                "verified": false,
                "summary": {
                    "id": id,
                    "state": "closed-late",
                    "next_action": CLOSED_LATE_NEXT_ACTION,
                    "handoff": {
                        "readiness": "safe",
                        "reason": CLOSED_LATE_HANDOFF,
                    },
                },
            }));
        }
        _ => {
            println!("{id}");
            println!("  State: closed late; never re-checked, not verified");
            println!("  {sentence}");
            println!("  Next: {CLOSED_LATE_NEXT_ACTION}");
            println!("  Handoff: safe; {CLOSED_LATE_HANDOFF}");
        }
    }
    Some(Ok(()))
}

/// The report `change status`, `show` and `ship-status` give for a closed change, or `None` when
/// `id` resolves to a live or archived workspace (REQ-change-159).
fn closed_report(
    root: &Path,
    id: &str,
    format: OutputFormat,
    phase: &str,
) -> Option<Result<(), String>> {
    if !change::is_only_closed(root, id) {
        return None;
    }
    if let Err(error) = change::check_read_deadline(phase, 0, 1) {
        return Some(Err(error));
    }
    let (package, record) = change::closed_record(root, id)?;
    let sentence = change::closed_sentence(root, id)?;
    let package = package
        .strip_prefix(root)
        .unwrap_or(&package)
        .to_string_lossy()
        .replace('\\', "/");
    match format {
        OutputFormat::Json => {
            let closed = match &record {
                Ok(record) => serde_json::to_value(record).unwrap_or(serde_json::Value::Null),
                Err(error) => serde_json::json!({ "error": error, "reviewed": false }),
            };
            print_json(&serde_json::json!({
                "id": id,
                "closed": closed,
                "package": package,
                "reviewed": false,
                "summary": {
                    "id": id,
                    "state": "closed",
                    "next_action": CLOSED_NEXT_ACTION,
                    "handoff": {
                        "readiness": "safe",
                        "reason": CLOSED_HANDOFF,
                    },
                },
            }));
        }
        _ => {
            println!("{id}");
            println!("  State: closed; never reviewed");
            println!("  {sentence}");
            println!("  Next: {CLOSED_NEXT_ACTION}");
            println!("  Handoff: safe; {CLOSED_HANDOFF}");
        }
    }
    Some(Ok(()))
}

/// The next action status gives a closed change: there is none.
const CLOSED_NEXT_ACTION: &str = "none: a closed change is never reviewed or finalized";
/// Why a session may clear its context after reading a closed change.
const CLOSED_HANDOFF: &str = "the change was closed; nothing this session knows is still needed";

/// The next action status gives a closed-late change: there is none.
const CLOSED_LATE_NEXT_ACTION: &str = "none: a closed-late change is never re-checked or finalized";
/// Why a session may clear its context after reading a closed-late change.
const CLOSED_LATE_HANDOFF: &str =
    "the change was closed late; nothing this session knows is still needed";

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

    /// #923 and #953 (REQ-cmd-change-036): the printed resume command must run exactly the
    /// arguments it names. Splitting it with `sh` gives back every original argument,
    /// metacharacters and all, with and without `SPECSYNC_TIMEOUT` in the environment (the test
    /// passes that in, so it does not depend on the environment the suite runs in).
    #[test]
    fn the_resume_command_splits_back_into_the_original_arguments() {
        let original = [
            "--root",
            "/tmp/a$HOME;rm -rf x*",
            "change",
            "status",
            "it's",
            "",
            "two words",
            "--json",
        ];
        for timeout_env in [false, true] {
            let resume = resume_command_for(
                original
                    .iter()
                    .map(|argument| (*argument).to_string())
                    .chain(["--timeout".to_string(), "5".to_string()]),
                timeout_env,
            );
            let (prefix, leading) = if timeout_env {
                ("SPECSYNC_TIMEOUT= specsync ", 2)
            } else {
                ("specsync ", 1)
            };
            assert!(resume.starts_with(prefix), "{resume}");
            assert!(!resume.contains("--timeout"), "{resume}");
            let output = std::process::Command::new("sh")
                .arg("-c")
                .arg(format!(
                    "set -- {resume}; shift {leading}; for a in \"$@\"; do printf '%s\\0' \"$a\"; done"
                ))
                .output()
                .unwrap();
            assert!(output.status.success(), "{resume}");
            let split: Vec<String> = String::from_utf8(output.stdout)
                .unwrap()
                .split('\0')
                .map(str::to_string)
                .collect();
            let mut expected: Vec<String> = original
                .iter()
                .map(|argument| (*argument).to_string())
                .collect();
            expected.push(String::new());
            assert_eq!(
                split, expected,
                "the resume command {resume} must name exactly the original arguments"
            );
        }
    }

    /// The floor must be WIRED, not merely present (#533).
    ///
    /// `floor_sequence_ledger_to_committed` has its own unit tests, but those
    /// exercise the function directly. Nothing asserted that the lifecycle
    /// staging path (`git_commit_lifecycle`, formerly `git_commit_all`)
    /// actually calls it, so deleting the call left the entire suite green
    /// while every lifecycle commit went back to staging a stale ledger over a
    /// higher committed mark — the exact regression #533 is about.
    ///
    /// This test drives the real staging path and inspects what landed in the
    /// commit, so it fails if the call is removed.
    #[test]
    fn lifecycle_commit_raises_a_stale_ledger_before_staging_it() {
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

        // Nothing is owned here: the ledger is tracked, so the tracked-edit rule stages it.
        let scope = change::LifecycleCommitScope {
            owned: Vec::new(),
            runtime: Vec::new(),
        };
        git_commit_lifecycle(root, &scope, "lifecycle commit", true).expect("commit");

        assert_eq!(
            committed_sequence(),
            3,
            "the staging path must raise the stale ledger before staging it; \
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
        let questions = change::next_questions_for(root, &record);
        assert!(questions.is_empty(), "interview must be complete");

        for surface in ["status", "show", "list"] {
            let next = text_mode_next_action(root, &record);
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

    // #690: the product-tip stage must never advertise a command its own state machine
    // refuses. For each state whose product stage is not done, the advertised `specsync change`
    // verb must be one whose guard admits that state.
    #[test]
    fn ship_stage_product_action_is_accepted_by_the_state_machine() {
        let temp = TempDir::new().expect("temp project");
        let root = temp.path();
        let mut record = draft_fixture(root);
        let tip = HeadTip {
            tip_class: "product".into(),
            tip_sha: Some("0".repeat(40)),
            parent_sha: None,
        };
        let admitted = |verb: &str, state: ChangeState| match verb {
            "answer" | "approve" => state == ChangeState::Draft,
            "check" => matches!(
                state,
                ChangeState::Approved | ChangeState::Implementing | ChangeState::Verifying
            ),
            "reopen" => matches!(state, ChangeState::Accepted | ChangeState::Archived),
            other => panic!("unexpected advertised verb `{other}`"),
        };
        for state in [
            ChangeState::Draft,
            ChangeState::Approved,
            ChangeState::Implementing,
            ChangeState::Verifying,
            ChangeState::Accepted,
            ChangeState::Archived,
        ] {
            record.state = state;
            // Verification present but no longer current for this content.
            let stages = ship_stages(
                &record,
                &tip,
                Some(&"1".repeat(40)),
                true,
                false,
                ShipReviewStatus::Missing,
                false,
            );
            let product = stages
                .iter()
                .find(|stage| stage["id"] == "product_tip")
                .expect("product stage");
            assert_eq!(product["status"], "current", "{state:?}");
            let action = product["action"].as_str().unwrap();
            let verb = action
                .split("specsync change ")
                .nth(1)
                .and_then(|rest| rest.split_whitespace().next())
                .unwrap_or_else(|| panic!("{state:?}: no `specsync change` command in {action}"));
            assert!(
                admitted(verb, state),
                "{state:?}: product stage advertises `{verb}`, which refuses that state: {action}"
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

    /// Give a fixture repository a remote default branch at its current commit, so the
    /// ownership recovery can tell this branch's work from untouched files (#1000, #1037).
    fn with_remote_default(root: &Path) {
        let remote = root.parent().unwrap().join("remote.git");
        // The remote's HEAD names `main` whatever `init.defaultBranch` says, so a clone of it
        // checks out the pushed tree on stock git too (#1037 review).
        let status = std::process::Command::new("git")
            .args([
                "init",
                "-q",
                "--bare",
                "--initial-branch=main",
                remote.to_str().unwrap(),
            ])
            .status()
            .unwrap();
        assert!(status.success());
        git_in(root, &["remote", "add", "origin", remote.to_str().unwrap()]);
        git_in(root, &["push", "-q", "origin", "main"]);
        git_in(root, &["remote", "set-head", "origin", "main"]);
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

    /// A directory-scoped no-spec-change change that delivers `src/billing/charge.ts`, which
    /// `billing` owns, and `src/billing/orphan.ts`, which no module owns: approved, checked and
    /// reviewed at HEAD, so only canonical ownership stands between it and finalize (#986, #956).
    fn reviewed_unowned_delivery_fixture(root: &Path) -> String {
        reviewed_delivery_fixture(root, "billing", "src/billing/", &["src/billing/orphan.ts"])
    }

    /// The general form: `module`'s canonical spec owns `src/billing/charge.ts`; a
    /// no-spec-change change scoped to `scope` edits it and adds `extra` files.
    fn reviewed_delivery_fixture(root: &Path, module: &str, scope: &str, extra: &[&str]) -> String {
        git_project_fixture(root);
        fs::create_dir_all(root.join(format!("specs/{module}"))).unwrap();
        fs::write(
            root.join(format!("specs/{module}/{module}.spec.md")),
            format!("---\nmodule: {module}\nversion: 1\nstatus: stable\nfiles:\n  - src/billing/charge.ts\n---\n\n# Billing\n\n## Purpose\n\nCharges.\n\n## Public API\n\nNone.\n\n## Invariants\n\nStable.\n\n## Behavioral Examples\n\nWorks.\n\n## Error Cases\n\nNone.\n\n## Dependencies\n\nNone.\n\n## Change Log\n\n| Date | Change |\n|------|--------|\n| 2026-01-01 | Initial |\n"),
        )
        .unwrap();
        fs::create_dir_all(root.join("src/billing")).unwrap();
        fs::write(root.join("src/billing/charge.ts"), "const rate = 1;\n").unwrap();
        git_in(root, &["add", "."]);
        git_in(root, &["commit", "-m", "billing"]);
        let record = change::create_change(
            root,
            CreateChangeRequest {
                description: "Tune billing".into(),
                kind: ChangeKind::BugFix,
                affected_specs: Vec::new(),
                affected_paths: vec![scope.into()],
                requested_artifacts: Vec::new(),
                no_spec_change: true,
                rationale: Some("Internal billing details".into()),
            },
        )
        .expect("create draft");
        let id = record.id.clone();
        approve_and_implement_fixture(root, &id);
        fs::write(root.join("src/billing/charge.ts"), "const rate = 2;\n").unwrap();
        for path in extra {
            let path = root.join(path);
            fs::create_dir_all(path.parent().unwrap()).unwrap();
            fs::write(path, "const spare = 0;\n").unwrap();
        }
        git_in(root, &["add", "."]);
        git_in(root, &["commit", "-m", "implement"]);
        change::check_change(root, Some(&id)).expect("check");
        git_in(root, &["add", "."]);
        git_in(root, &["commit", "-m", "record verification"]);
        // A passing review refuses a change finalize would refuse on ownership (#1002), so a
        // fixture that is unowned by design stays verifying without a review.
        match change::record_scoped_review_with_verdict(
            root,
            &id,
            "Independent".into(),
            change::ScopedReviewVerdict::Pass,
        ) {
            Ok(_) => {}
            Err(error) if error.contains("scoped review cannot pass") => {}
            Err(error) => panic!("review: {error}"),
        }
        id
    }

    /// #1000: an unowned input the change never modified is in scope only because the prefix is
    /// wider than the work, so the recovery says to narrow `affected_paths` instead of declaring
    /// that owner; a modified unowned input keeps the declare-the-owner recovery.
    #[test]
    fn the_ownership_recovery_narrows_a_scope_over_files_the_change_never_touched() {
        let temp = tempfile::TempDir::new().unwrap();
        let root = temp.path().join("work");
        fs::create_dir_all(&root).unwrap();
        git_project_fixture(&root);
        for (module, file) in [
            ("billing", "src/billing/charge.ts"),
            ("ledger", "src/billing/ledger.ts"),
            ("tarifa", "src/billing/tarifa-ñ.ts"),
        ] {
            fs::create_dir_all(root.join(format!("specs/{module}"))).unwrap();
            fs::write(
                root.join(format!("specs/{module}/{module}.spec.md")),
                format!("---\nmodule: {module}\nversion: 1\nstatus: stable\nfiles:\n  - {file}\n---\n\n# {module}\n\n## Purpose\n\nOwner.\n\n## Public API\n\nNone.\n\n## Invariants\n\nStable.\n\n## Behavioral Examples\n\nWorks.\n\n## Error Cases\n\nNone.\n\n## Dependencies\n\nNone.\n\n## Change Log\n\n| Date | Change |\n|------|--------|\n| 2026-01-01 | Initial |\n"),
            )
            .unwrap();
            fs::create_dir_all(root.join(file).parent().unwrap()).unwrap();
            fs::write(root.join(file), "const rate = 1;\n").unwrap();
        }
        git_in(&root, &["add", "."]);
        git_in(&root, &["commit", "-m", "billing and ledger"]);
        with_remote_default(&root);
        let record = change::create_change(
            &root,
            CreateChangeRequest {
                description: "Tune billing charges".into(),
                kind: ChangeKind::BugFix,
                affected_specs: Vec::new(),
                affected_paths: vec!["src/billing/".into()],
                requested_artifacts: Vec::new(),
                no_spec_change: true,
                rationale: Some("Internal billing details".into()),
            },
        )
        .expect("create draft");
        let id = record.id.clone();
        approve_and_implement_fixture(&root, &id);
        fs::write(root.join("src/billing/charge.ts"), "const rate = 2;\n").unwrap();
        // A modified file whose name is not ASCII: git C-quotes it unless asked for raw names.
        fs::write(root.join("src/billing/tarifa-ñ.ts"), "const rate = 3;\n").unwrap();
        // A new file the change adds that no spec lists: it needs a `files:` entry, not a module.
        fs::write(root.join("src/billing/helper.ts"), "const help = 1;\n").unwrap();
        git_in(&root, &["add", "."]);
        git_in(&root, &["commit", "-m", "implement"]);
        change::check_change(&root, Some(&id)).expect("check");

        let record = change::load_change(&root, &id).unwrap();
        let (blocker, recovery) = change::ownership_preflight_blocker(&root, &record)
            .unwrap()
            .expect("both billing files are unowned by the declared modules");
        assert!(
            blocker.contains("src/billing/charge.ts") && blocker.contains("src/billing/ledger.ts"),
            "{blocker}"
        );
        assert!(
            recovery
                .contains("src/billing/ledger.ts is not modified by this change since its base")
                && recovery.contains(
                "affected_paths 'src/billing/charge.ts,src/billing/helper.ts,src/billing/tarifa-ñ.ts'"
            ),
            "the untouched file asks for a narrower scope: {recovery}"
        );
        assert!(
            recovery.contains("affected_specs 'billing,tarifa'") && !recovery.contains("ledger'"),
            "every modified file's owner is declared, including a non-ASCII one: {recovery}"
        );
        assert!(
            !recovery.contains("tarifa-ñ.ts is not modified")
                && !blocker.contains("tarifa-ñ.ts: owned by"),
            "a modified non-ASCII file is not narrowed out: {recovery}"
        );
        assert_eq!(
            recovery.matches("change send-back").count(),
            1,
            "one send-back rescopes both sets in the same draft: {recovery}"
        );
        assert!(
            recovery.contains("and add src/billing/helper.ts to its owning spec's `files:` list"),
            "the mixed recovery keeps the files: repair for a modified file no spec lists: {recovery}"
        );
        assert!(
            blocker.contains("src/billing/ledger.ts: owned by `ledger`, but not modified"),
            "the untouched hint still names its owner: {blocker}"
        );
        assert!(
            blocker.contains("src/billing/ledger.ts: owned by `ledger`, but not modified by this change since its base"),
            "the blocker hint for an untouched file agrees with the recovery: {blocker}"
        );

        // A base commit that is not a full commit id is never handed to Git: a value shaped
        // like an option writes nothing. Without a usable base, as without any base, nothing is
        // classified untouched, so ledger.ts keeps the declare recovery too.
        let probe = root.join("probe-output.txt");
        let mut tampered = record.clone();
        tampered.base_commit = Some(format!("--output={}", probe.display()));
        let mut baseless = record.clone();
        baseless.base_commit = None;
        for unusable in [&tampered, &baseless] {
            let (_, recovery) = change::ownership_preflight_blocker(&root, unusable)
                .unwrap()
                .expect("still unowned");
            assert!(
                !recovery.contains("not modified by this change")
                    && recovery.contains("affected_specs 'billing,ledger,tarifa'"),
                "without a usable base every owner is declared: {recovery}"
            );
        }
        assert!(
            !probe.exists(),
            "the record's base_commit never became a Git option"
        );

        // A file the change adds and has not committed yet is its work too (the untracked-files
        // arm), non-ASCII name included: it stays in the narrowed scope, and only ledger.ts is
        // narrowed out.
        fs::write(root.join("src/billing/recargo-é.ts"), "const fee = 1;\n").unwrap();
        let (_, recovery) = change::ownership_preflight_blocker(&root, &record)
            .unwrap()
            .expect("still unowned");
        assert!(
            recovery.contains("(src/billing/ledger.ts is not modified by this change since its base")
                && recovery.contains(
                    "affected_paths 'src/billing/charge.ts,src/billing/helper.ts,src/billing/recargo-é.ts,src/billing/tarifa-ñ.ts'"
                ),
            "an untracked file the change adds is not narrowed out: {recovery}"
        );
        fs::remove_file(root.join("src/billing/recargo-é.ts")).unwrap();

        // An untouched file whose owner the answer already declares is fixed by that answer,
        // so it is not narrowed out as well.
        let mut billing = fs::read_to_string(root.join("specs/billing/billing.spec.md")).unwrap();
        billing = billing.replace(
            "  - src/billing/charge.ts\n",
            "  - src/billing/charge.ts\n  - src/billing/ledger.ts\n",
        );
        fs::write(root.join("specs/billing/billing.spec.md"), billing).unwrap();
        fs::remove_dir_all(root.join("specs/ledger")).unwrap();
        let (_, recovery) = change::ownership_preflight_blocker(&root, &record)
            .unwrap()
            .expect("billing's files are still undeclared");
        assert!(
            !recovery.contains("not modified by this change"),
            "declaring billing already covers ledger.ts: {recovery}"
        );
    }

    /// #1000: when every unowned input is untouched (the edited file's owner is declared), the
    /// recovery only narrows the scope: one send-back, then the quoted `affected_paths` answer.
    #[test]
    fn the_ownership_recovery_only_narrows_when_every_unowned_input_is_untouched() {
        let temp = tempfile::TempDir::new().unwrap();
        let root = temp.path().join("work");
        fs::create_dir_all(&root).unwrap();
        git_project_fixture(&root);
        for (module, file) in [
            ("billing", "src/billing/charge.ts"),
            ("ledger", "src/billing/ledger.ts"),
        ] {
            fs::create_dir_all(root.join(format!("specs/{module}"))).unwrap();
            fs::write(
                root.join(format!("specs/{module}/{module}.spec.md")),
                format!("---\nmodule: {module}\nversion: 1\nstatus: stable\nfiles:\n  - {file}\n---\n\n# {module}\n\n## Purpose\n\nOwner.\n\n## Public API\n\nNone.\n\n## Invariants\n\nStable.\n\n## Behavioral Examples\n\nWorks.\n\n## Error Cases\n\nNone.\n\n## Dependencies\n\nNone.\n\n## Change Log\n\n| Date | Change |\n|------|--------|\n| 2026-01-01 | Initial |\n"),
            )
            .unwrap();
            fs::create_dir_all(root.join(file).parent().unwrap()).unwrap();
            fs::write(root.join(file), "const rate = 1;\n").unwrap();
        }
        git_in(&root, &["add", "."]);
        fs::write(root.join("src/billing/notes.md"), "notes\n").unwrap();
        git_in(&root, &["add", "."]);
        git_in(&root, &["commit", "-m", "billing and ledger"]);
        with_remote_default(&root);
        let record = change::create_change(
            &root,
            CreateChangeRequest {
                description: "Tune billing charges only".into(),
                kind: ChangeKind::BugFix,
                affected_specs: vec!["billing".into()],
                affected_paths: vec!["src/billing/".into()],
                requested_artifacts: Vec::new(),
                no_spec_change: true,
                rationale: Some("Internal billing details".into()),
            },
        )
        .expect("create draft");
        let id = record.id.clone();
        approve_and_implement_fixture(&root, &id);
        fs::write(root.join("src/billing/charge.ts"), "const rate = 2;\n").unwrap();
        // The change also deletes a file in its scope: the narrowed scope must still name it, or
        // the shipped change fails the meaningful-path audit, `change audit --strict` (#1037
        // review).
        git_in(&root, &["rm", "-q", "src/billing/notes.md"]);
        git_in(&root, &["add", "."]);
        git_in(&root, &["commit", "-m", "implement"]);
        // Someone else edits ledger.ts on the default branch, and this branch merges it: the
        // upstream edit is not this change's work, so ledger.ts stays untouched (#1037 review).
        let upstream = temp.path().join("upstream");
        let remote = temp.path().join("remote.git");
        let status = std::process::Command::new("git")
            .args([
                "clone",
                "-q",
                remote.to_str().unwrap(),
                upstream.to_str().unwrap(),
            ])
            .status()
            .unwrap();
        assert!(status.success());
        git_in(&upstream, &["config", "user.email", "u@example.com"]);
        git_in(&upstream, &["config", "user.name", "U"]);
        fs::write(upstream.join("src/billing/ledger.ts"), "const rate = 3;\n").unwrap();
        git_in(&upstream, &["commit", "-q", "-am", "upstream ledger edit"]);
        git_in(&upstream, &["push", "-q", "origin", "main"]);
        git_in(&root, &["fetch", "-q", "origin"]);
        git_in(&root, &["merge", "-q", "--no-edit", "origin/main"]);
        change::check_change(&root, Some(&id)).expect("check");

        let record = change::load_change(&root, &id).unwrap();
        let (_, recovery) = change::ownership_preflight_blocker(&root, &record)
            .unwrap()
            .expect("ledger.ts is in scope and unowned by the declared module");
        assert!(
            recovery
                .starts_with("src/billing/ledger.ts is not modified by this change since its base"),
            "{recovery}"
        );
        assert!(
            recovery.contains("affected_paths 'src/billing/charge.ts,src/billing/notes.md'")
                && !recovery.contains("declare the owners")
                && recovery.matches("change send-back").count() == 1,
            "narrow only, with one send-back: {recovery}"
        );
    }

    /// #1037 review: a file another active change on the branch covers is that change's work. The
    /// first change is told to narrow it out of its scope, not to declare its owner.
    #[test]
    fn the_ownership_recovery_leaves_a_sibling_changes_file_to_that_change() {
        let temp = tempfile::TempDir::new().unwrap();
        let root = temp.path().join("work");
        fs::create_dir_all(&root).unwrap();
        git_project_fixture(&root);
        for (module, file) in [
            ("billing", "src/billing/charge.ts"),
            ("ledger", "src/billing/ledger.ts"),
        ] {
            fs::create_dir_all(root.join(format!("specs/{module}"))).unwrap();
            fs::write(
                root.join(format!("specs/{module}/{module}.spec.md")),
                format!("---\nmodule: {module}\nversion: 1\nstatus: stable\nfiles:\n  - {file}\n---\n\n# {module}\n\n## Purpose\n\nOwner.\n\n## Public API\n\nNone.\n\n## Invariants\n\nStable.\n\n## Behavioral Examples\n\nWorks.\n\n## Error Cases\n\nNone.\n\n## Dependencies\n\nNone.\n\n## Change Log\n\n| Date | Change |\n|------|--------|\n| 2026-01-01 | Initial |\n"),
            )
            .unwrap();
            fs::create_dir_all(root.join(file).parent().unwrap()).unwrap();
            fs::write(root.join(file), "const rate = 1;\n").unwrap();
        }
        git_in(&root, &["add", "."]);
        git_in(&root, &["commit", "-m", "billing and ledger"]);
        with_remote_default(&root);
        let sibling = change::create_change(
            &root,
            CreateChangeRequest {
                description: "Fix the ledger rounding".into(),
                kind: ChangeKind::BugFix,
                affected_specs: vec!["ledger".into()],
                affected_paths: vec!["src/billing/ledger.ts".into()],
                requested_artifacts: Vec::new(),
                no_spec_change: true,
                rationale: Some("Internal ledger details".into()),
            },
        )
        .expect("create the sibling draft");
        let record = change::create_change(
            &root,
            CreateChangeRequest {
                description: "Tune billing charges only".into(),
                kind: ChangeKind::BugFix,
                affected_specs: vec!["billing".into()],
                affected_paths: vec!["src/billing/".into()],
                requested_artifacts: Vec::new(),
                no_spec_change: true,
                rationale: Some("Internal billing details".into()),
            },
        )
        .expect("create draft");
        let id = record.id.clone();
        approve_and_implement_fixture(&root, &id);
        fs::write(root.join("src/billing/charge.ts"), "const rate = 2;\n").unwrap();
        fs::write(root.join("src/billing/ledger.ts"), "const rate = 3;\n").unwrap();
        git_in(&root, &["add", "."]);
        git_in(&root, &["commit", "-m", "implement both"]);

        let record = change::load_change(&root, &id).unwrap();
        let (_, recovery) = change::ownership_preflight_blocker(&root, &record)
            .unwrap()
            .expect("ledger.ts is in this change's scope and unowned by its declared module");
        assert!(
            recovery.contains("affected_paths 'src/billing/charge.ts'")
                && !recovery.contains("declare the owners")
                && !recovery.contains("affected_specs"),
            "the sibling `{}` covers ledger.ts, so this change narrows it out: {recovery}",
            sibling.id
        );
    }

    /// #1037 review: a change whose `src/billing/` scope edits `charge.ts` (owned by the declared
    /// `billing`) and `ledger.ts` (owned by the undeclared `ledger`), committing its records with
    /// the code when `commit_records`. Returns the temp dir, the project root and the change id.
    fn edited_owner_fixture(commit_records: bool) -> (tempfile::TempDir, PathBuf, String) {
        let temp = tempfile::TempDir::new().unwrap();
        let root = temp.path().join("work");
        fs::create_dir_all(&root).unwrap();
        git_project_fixture(&root);
        for (module, file) in [
            ("billing", "src/billing/charge.ts"),
            ("ledger", "src/billing/ledger.ts"),
        ] {
            fs::create_dir_all(root.join(format!("specs/{module}"))).unwrap();
            fs::write(
                root.join(format!("specs/{module}/{module}.spec.md")),
                format!("---\nmodule: {module}\nversion: 1\nstatus: stable\nfiles:\n  - {file}\n---\n\n# {module}\n\n## Purpose\n\nOwner.\n\n## Public API\n\nNone.\n\n## Invariants\n\nStable.\n\n## Behavioral Examples\n\nWorks.\n\n## Error Cases\n\nNone.\n\n## Dependencies\n\nNone.\n\n## Change Log\n\n| Date | Change |\n|------|--------|\n| 2026-01-01 | Initial |\n"),
            )
            .unwrap();
            fs::create_dir_all(root.join(file).parent().unwrap()).unwrap();
            fs::write(root.join(file), "const rate = 1;\n").unwrap();
        }
        git_in(&root, &["add", "."]);
        git_in(&root, &["commit", "-m", "billing and ledger"]);
        with_remote_default(&root);
        let record = change::create_change(
            &root,
            CreateChangeRequest {
                description: "Tune billing charges".into(),
                kind: ChangeKind::BugFix,
                affected_specs: vec!["billing".into()],
                affected_paths: vec!["src/billing/".into()],
                requested_artifacts: Vec::new(),
                no_spec_change: true,
                rationale: Some("Internal billing details".into()),
            },
        )
        .expect("create draft");
        approve_and_implement_fixture(&root, &record.id);
        fs::write(root.join("src/billing/charge.ts"), "const rate = 2;\n").unwrap();
        fs::write(root.join("src/billing/ledger.ts"), "const rate = 2;\n").unwrap();
        git_in(&root, &["add", if commit_records { "." } else { "src" }]);
        git_in(&root, &["commit", "-m", "implement both"]);
        (temp, root, record.id)
    }

    /// The edited `ledger.ts` keeps the declare recovery and is never called untouched.
    fn assert_declares_ledger(root: &Path, id: &str, when: &str) {
        let record = change::load_change(root, id).unwrap();
        let (blocker, recovery) = change::ownership_preflight_blocker(root, &record)
            .unwrap()
            .expect("ledger.ts is edited and unowned by the declared module");
        assert!(
            recovery.contains("affected_specs 'billing,ledger'")
                && !recovery.contains("not modified by this change")
                && !blocker.contains("not modified by this change"),
            "{when}: the edited ledger.ts keeps the declare recovery: {recovery}"
        );
    }

    /// #1037 review: once the change's own commits are on the remote default branch (it merged
    /// with the change still active, the documented recovery state), the merge base no longer
    /// separates its work from upstream work. Nothing is classified then, so a file the change
    /// edited is never called untouched: not when HEAD is the merge base, and not after a later
    /// local commit on top, as a recovery `change check --commit` on the default branch makes.
    #[test]
    fn the_ownership_recovery_declares_every_owner_once_the_change_reached_the_default_branch() {
        let (_temp, root, id) = edited_owner_fixture(true);
        assert_declares_ledger(&root, &id, "before the work reaches the default branch");

        // The branch merges with the change still active: origin/main is HEAD.
        git_in(&root, &["push", "-q", "origin", "HEAD:main"]);
        git_in(&root, &["fetch", "-q", "origin"]);
        assert_declares_ledger(&root, &id, "with HEAD at the merge base");

        // A later local commit on top: the merge base holds the change's own workspace.
        fs::write(root.join("NOTES.md"), "recovery notes\n").unwrap();
        git_in(&root, &["add", "."]);
        git_in(
            &root,
            &["commit", "-m", "recovery commit on the default branch"],
        );
        assert_declares_ledger(&root, &id, "after a local commit past the merge base");
    }

    /// #1037 review: the same holds when the change's records were never committed, so the
    /// merge base cannot hold them: HEAD at the merge base past the change's base is enough.
    #[test]
    fn the_ownership_recovery_declares_every_owner_when_head_is_the_merge_base_past_the_base() {
        let (_temp, root, id) = edited_owner_fixture(false);
        assert_declares_ledger(&root, &id, "before the work reaches the default branch");
        git_in(&root, &["push", "-q", "origin", "HEAD:main"]);
        git_in(&root, &["fetch", "-q", "origin"]);
        assert_declares_ledger(
            &root,
            &id,
            "with HEAD at the merge base and the records uncommitted",
        );
    }

    /// Codex P2 on #999: the recovery repairs every unowned input, not only the ones a module
    /// claims: `billing` is declared by the answer, and the path no module owns is named for a
    /// `files:` entry.
    #[test]
    fn the_ownership_recovery_names_a_repair_for_every_unowned_input() {
        let temp = tempfile::TempDir::new().unwrap();
        let root = temp.path().join("work");
        fs::create_dir_all(&root).unwrap();
        let id = reviewed_unowned_delivery_fixture(&root);
        let record = change::load_change(&root, &id).unwrap();
        let (blocker, recovery) = change::ownership_preflight_blocker(&root, &record)
            .unwrap()
            .expect("both inputs lack a declared owner");
        assert!(
            blocker.contains("src/billing/charge.ts") && blocker.contains("src/billing/orphan.ts"),
            "{blocker}"
        );
        assert!(
            recovery.contains(&format!(
                "specsync change answer '{id}' affected_specs 'billing'"
            )),
            "{recovery}"
        );
        assert!(
            recovery.contains("src/billing/orphan.ts") && recovery.contains("`files:`"),
            "the path no module owns needs its own repair: {recovery}"
        );
    }

    /// Codex P2 on #999: `change status`, `change pr` and the Archive check take their ship advice
    /// from `archive_candidate_can_ship`, so it must not call a change ready that ship refuses on
    /// canonical ownership.
    #[test]
    fn an_archive_candidate_with_unowned_production_source_is_not_told_to_ship() {
        let temp = tempfile::TempDir::new().unwrap();
        let root = temp.path().join("work");
        fs::create_dir_all(&root).unwrap();
        let id = reviewed_unowned_delivery_fixture(&root);
        let workspace = root.join(".specsync/changes").join(&id);
        let record = change::load_change(&root, &id).unwrap();
        assert_eq!(record.state, ChangeState::Verifying);

        // #1002: the review itself refuses to pass, so the Archive check (which cannot prove
        // ownership) never sees a passing review at HEAD for this change.
        let refused = change::record_scoped_review_with_verdict(
            &root,
            &id,
            "Independent".into(),
            change::ScopedReviewVerdict::Pass,
        )
        .expect_err("a passing review refuses a change finalize would refuse on ownership");
        assert!(
            refused.contains("scoped review cannot pass")
                && refused
                    .contains("finalize would refuse production source no declared module owns")
                && refused.contains("Recovery:"),
            "{refused}"
        );
        assert!(
            !workspace.join("review.json").exists(),
            "nothing is recorded"
        );
        assert!(!archive_candidate_can_ship(&root, &record, &workspace));

        // A passing review recorded before #1002 refused this is still not told to ship.
        let head = git_output_text(&root, &["rev-parse", "HEAD"]).unwrap();
        fs::write(
            workspace.join("review.json"),
            format!(
                "{{\"verdict\":\"pass\",\"implementation_commit\":\"{}\"}}\n",
                head.trim()
            ),
        )
        .unwrap();
        assert!(
            !archive_candidate_can_ship(&root, &record, &workspace),
            "ship refuses unowned production source, so the candidate is not told to ship"
        );
    }

    /// Codex P2 on #999: a reserved delivery path (here `.github/scripts/check.py`, production
    /// source once `source_dirs` includes `.`) is signed with the exact delivery owner, which
    /// finalize accepts, so the preflight must not call it unowned.
    #[test]
    fn reserved_delivery_paths_are_not_ownership_blockers() {
        let temp = tempfile::TempDir::new().unwrap();
        let root = temp.path().join("work");
        fs::create_dir_all(root.join(".specsync")).unwrap();
        fs::write(
            root.join(".specsync/config.toml"),
            "specs_dir = \"specs\"\nsource_dirs = [\".\"]\n",
        )
        .unwrap();
        let id = reviewed_delivery_fixture(
            &root,
            "billing",
            ".github/scripts/",
            &[".github/scripts/check.py"],
        );
        let record = change::load_change(&root, &id).unwrap();
        assert_eq!(
            change::ownership_preflight_blocker(&root, &record).unwrap(),
            None,
            "finalize signs reserved delivery paths with the exact delivery owner"
        );
    }

    /// Codex P1 on #999: the recovery quotes every word it interpolates, so a module name with
    /// shell syntax reaches `change answer` as one argument and runs nothing.
    #[test]
    fn the_ownership_recovery_quotes_every_interpolated_word() {
        let temp = tempfile::TempDir::new().unwrap();
        let root = temp.path().join("work");
        fs::create_dir_all(&root).unwrap();
        let module = "billing;touch PWN";
        let id = reviewed_delivery_fixture(&root, module, "src/billing/", &[]);
        let record = change::load_change(&root, &id).unwrap();
        let (_, recovery) = change::ownership_preflight_blocker(&root, &record)
            .unwrap()
            .expect("charge.ts is owned by an undeclared module");
        let sandbox = tempfile::TempDir::new().unwrap();
        // `<name>` is a placeholder a person fills in (and a redirection to `sh`), so only the
        // placeholder-free commands are run; the send-back's id is checked as text.
        assert!(
            recovery.contains(&format!("specsync change send-back '{id}' --actor <name>")),
            "{recovery}"
        );
        let argv: Vec<Vec<String>> = recovery
            .split('`')
            .skip(1)
            .step_by(2)
            .filter(|command| !command.contains("<name>"))
            .map(|command| {
                let (_, arguments) = command.split_once(' ').unwrap();
                let output = std::process::Command::new("sh")
                    .arg("-c")
                    .arg(format!("printf '%s\\n' {arguments}"))
                    .current_dir(sandbox.path())
                    .output()
                    .unwrap();
                assert!(output.status.success(), "sh rejected `{command}`");
                String::from_utf8(output.stdout)
                    .unwrap()
                    .lines()
                    .map(str::to_string)
                    .collect()
            })
            .collect();
        assert_eq!(
            fs::read_dir(sandbox.path()).unwrap().count(),
            0,
            "{recovery}"
        );
        assert_eq!(
            argv.last().unwrap(),
            &vec![
                "change".to_string(),
                "answer".into(),
                id.clone(),
                "affected_specs".into(),
                module.into()
            ],
            "{recovery}"
        );
    }

    /// #902: a copy of a change that the remote default already archived is not an archive
    /// candidate to ship, even with a passing review at `HEAD` and its verification (the review
    /// predates the upstream archive). The pull-request sentence that `change pr` and the
    /// Archive check print then names `change status`, which carries #889's recovery.
    #[test]
    fn an_upstream_archived_copy_is_not_told_to_ship() {
        let temp = tempfile::TempDir::new().unwrap();
        let root = temp.path().join("work");
        fs::create_dir_all(&root).unwrap();
        git_project_fixture(&root);
        let id = reviewed_change_fixture(&root);
        let workspace = root.join(".specsync/changes").join(&id);
        let reviewed = git_rev_parse(&root, "HEAD").unwrap();
        let record = change::load_change(&root, &id).unwrap();
        assert!(
            archive_candidate_can_ship(&root, &record, &workspace),
            "control: a passing review at HEAD with no upstream archive ships"
        );
        let review = fs::read(workspace.join("review.json")).unwrap();
        let attempts = fs::read(workspace.join("review-attempts.json")).unwrap();

        // Archive it, publish that as origin's default branch, then return to the reviewed copy.
        change::finalize_change(&root, &id).expect("finalize");
        git_in(&root, &["add", "--all"]);
        git_in(&root, &["commit", "-q", "-m", "archive", "--allow-empty"]);
        let archived = git_rev_parse(&root, "HEAD").unwrap();
        let origin = temp.path().join("origin.git");
        git_in(
            temp.path(),
            &[
                "init",
                "-q",
                "--bare",
                "-b",
                "main",
                origin.to_str().unwrap(),
            ],
        );
        git_in(
            &root,
            &["remote", "add", "origin", origin.to_str().unwrap()],
        );
        git_in(
            &root,
            &[
                "push",
                "-q",
                "origin",
                &format!("{archived}:refs/heads/main"),
            ],
        );
        git_in(&root, &["fetch", "-q", "origin"]);
        git_in(&root, &["remote", "set-head", "origin", "main"]);
        git_in(&root, &["checkout", "-q", "-B", "stale-copy", &reviewed]);
        fs::write(workspace.join("review.json"), &review).unwrap();
        fs::write(workspace.join("review-attempts.json"), &attempts).unwrap();

        let stale = change::load_change(&root, &id).unwrap();
        assert_eq!(stale.state, ChangeState::Verifying);
        assert!(
            change::upstream_archive_of(&root, &stale).is_some(),
            "fixture: origin/main must hold an archive HEAD does not contain"
        );
        assert!(
            !archive_candidate_can_ship(&root, &stale, &workspace),
            "an upstream-archived copy must not be told to ship"
        );
        let sentence = pull_request_sentence(&root).unwrap();
        assert!(
            sentence.contains(&format!("Run `specsync change status {id}`")),
            "{sentence}"
        );
        assert!(!sentence.contains("specsync change ship"), "{sentence}");
    }

    /// Drives one change to `Verifying` with committed verification and a current scoped
    /// review — the exact state `ship-status` claims is ready and `finalize` is asked to
    /// honour. The caller owns the repository and the branch, because the three #743/#689
    /// cases differ only in what happens to history afterwards.
    fn reviewed_change_fixture(root: &Path) -> String {
        let id = approved_change_fixture(root);
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

    /// Drives one change through approval into implementation and stops there, with its
    /// workspace still uncommitted: the state `change check --commit` is run from.
    fn approved_change_fixture(root: &Path) -> String {
        let record = draft_fixture(root);
        let id = record.id.clone();
        approve_and_implement_fixture(root, &id);
        id
    }

    /// Answers the interview, fills the selected artifacts, approves, and starts implementation.
    fn approve_and_implement_fixture(root: &Path, id: &str) {
        for (question, answer) in [
            ("acceptance_criteria", "the fixture verifies"),
            ("public_contract", "no"),
            ("architecture_risk", "no"),
        ] {
            change::answer_question(root, id, question, answer).expect("answer");
        }
        // Fill whatever artifacts the adaptive interview selected; approval refuses on any
        // scaffold left incomplete.
        let workspace = root.join(".specsync/changes").join(id);
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
        change::approve_definition(root, id, Some("Reviewer".into()), None).expect("approve");
        change::start_implementation(root, id).expect("implement");
    }

    /// The delta-bearing sibling of `approved_change_fixture`: an implementing change whose
    /// `auth` semantic delta was approved with the wording it holds now.
    fn approved_delta_change_fixture(root: &Path) -> String {
        change::write_default_policy(root, Vec::new()).expect("write default policy");
        fs::create_dir_all(root.join("src")).unwrap();
        fs::create_dir_all(root.join("specs/auth")).unwrap();
        fs::write(root.join("src/auth.rs"), "pub fn authenticate() {}\n").unwrap();
        fs::write(
            root.join("specs/auth/auth.spec.md"),
            "---\nmodule: auth\nversion: 1.0.0\nstatus: stable\nfiles:\n  - src/auth.rs\n---\n\n# Auth\n\n## Purpose\n\nAuth.\n\n## Public API\n\nNone.\n\n## Invariants\n\nStable.\n\n## Behavioral Examples\n\nWorks.\n\n## Error Cases\n\nNone.\n\n## Dependencies\n\nNone.\n\n## Change Log\n\n| Date | Change |\n|------|--------|\n| 2026-01-01 | Initial |\n",
        )
        .unwrap();
        let record = change::create_change(
            root,
            CreateChangeRequest {
                description: "Reword the auth purpose".into(),
                kind: ChangeKind::Documentation,
                affected_specs: vec!["auth".into()],
                affected_paths: vec!["src/auth.rs".into()],
                requested_artifacts: Vec::new(),
                no_spec_change: false,
                rationale: None,
            },
        )
        .expect("create draft");
        fs::write(
            approved_delta_path(root, &record.id),
            "## MODIFIED\n\n### SPEC SECTION Purpose\n\nAuth tracks credentials.\n",
        )
        .unwrap();
        approve_and_implement_fixture(root, &record.id);
        record.id
    }

    fn approved_delta_path(root: &Path, id: &str) -> PathBuf {
        root.join(".specsync/changes")
            .join(id)
            .join("deltas/auth.md")
    }

    fn reapprove_action(id: &str) -> String {
        format!("run `specsync change approve {id} --actor <name>`")
    }

    /// DISCRIMINATOR for REQ-cmd-change-005 and REQ-change-111. On f240d37 text `Next:` says
    /// `change check` for a change whose approved delta was edited, and that check refuses.
    #[test]
    fn text_next_action_names_reapproval_when_an_approved_delta_was_edited() {
        let temp = TempDir::new().expect("temp project");
        let root = temp.path();
        let id = approved_delta_change_fixture(root);
        let delta = approved_delta_path(root, &id);
        let approved = fs::read_to_string(&delta).unwrap();
        fs::write(
            &delta,
            format!("{approved}\n## ADDED\n\n### REQUIREMENT REQ-auth-002\n\nThe system SHALL carry wording nobody approved.\n\nAcceptance Criteria\n- Nobody reviewed this.\n"),
        )
        .unwrap();
        let record = change::load_change(root, &id).expect("load");

        let next = text_mode_next_action(root, &record);

        assert_eq!(next, reapprove_action(&id));
        assert_eq!(
            next,
            change::summarize_change(root, &record).next_action,
            "text and JSON must name the same next action"
        );
        let refusal = change::check_change(root, Some(&id)).expect_err("check refuses the edit");
        assert!(refusal.contains("changed after approval"), "{refusal}");
    }

    /// DISCRIMINATOR for REQ-cmd-change-005, the pre-existing stale-scope case: JSON already
    /// asked for re-approval after a material scope change, while text `Next:` said `check`.
    #[test]
    fn text_next_action_names_reapproval_when_the_approved_scope_changed() {
        let temp = TempDir::new().expect("temp project");
        let root = temp.path();
        let id = approved_change_fixture(root);
        let state_path = root.join(".specsync/changes").join(&id).join("state.json");
        let mut state: serde_json::Value =
            serde_json::from_str(&fs::read_to_string(&state_path).unwrap()).unwrap();
        state["affected_paths"]
            .as_array_mut()
            .expect("affected paths")
            .push("docs/".into());
        fs::write(&state_path, serde_json::to_vec_pretty(&state).unwrap()).unwrap();
        let record = change::load_change(root, &id).expect("load");
        assert!(!change::summarize_change(root, &record).approval_valid);

        for lifecycle in [
            ChangeState::Approved,
            ChangeState::Implementing,
            ChangeState::Verifying,
        ] {
            let mut record = record.clone();
            record.state = lifecycle;
            assert_eq!(
                text_mode_next_action(root, &record),
                reapprove_action(&id),
                "{lifecycle:?}"
            );
        }
    }

    /// CONTROL, passes on f240d37 too: a current approval, with or without a delta, still
    /// sends text `Next:` to `change check`.
    #[test]
    fn text_next_action_names_check_for_a_current_approval() {
        for fixture in [approved_change_fixture, approved_delta_change_fixture] {
            let temp = TempDir::new().expect("temp project");
            let root = temp.path();
            let id = fixture(root);
            let record = change::load_change(root, &id).expect("load");
            assert_eq!(
                text_mode_next_action(root, &record),
                format!("run `specsync change check {id} --commit`")
            );
        }
    }

    /// Untracked files nobody asked the lifecycle to commit, shaped like the ones `git add -A`
    /// swept into pushed commits: a private debug archive, an agent's scratch directory, and an
    /// experiment script at the project root.
    const STRAYS: [&str; 3] = ["debug-dump.zip", ".agents/scratch.md", "exp2.sh"];

    fn write_strays(root: &Path) {
        for stray in STRAYS {
            let path = root.join(stray);
            fs::create_dir_all(path.parent().expect("parent")).expect("stray dir");
            fs::write(&path, format!("not for history: {stray}\n")).expect("stray");
        }
    }

    /// Every path any commit reachable from any ref in `git_dir` ever touched.
    fn paths_in_history(git_dir: &Path) -> Vec<String> {
        let output = std::process::Command::new("git")
            .arg(format!("--git-dir={}", git_dir.display()))
            .args(["log", "--all", "--name-only", "--pretty=format:"])
            .output()
            .expect("git log");
        assert!(output.status.success(), "git log failed");
        String::from_utf8_lossy(&output.stdout)
            .lines()
            .filter(|line| !line.is_empty())
            .map(str::to_owned)
            .collect()
    }

    fn worktree_status(root: &Path) -> Vec<String> {
        let output = std::process::Command::new("git")
            .args(["status", "--porcelain=v1", "--untracked-files=all"])
            .current_dir(root)
            .output()
            .expect("git status");
        String::from_utf8_lossy(&output.stdout)
            .lines()
            .map(str::to_owned)
            .collect()
    }

    /// The strays are in no commit of `git_dir` and still sit untracked in `root`, untouched.
    fn assert_strays_left_alone(root: &Path, git_dir: &Path) {
        let history = paths_in_history(git_dir);
        let status = worktree_status(root);
        for stray in STRAYS {
            assert!(
                !history.iter().any(|path| path == stray),
                "{stray} was committed; a lifecycle commit must never stage an untracked file the change does not own: {history:?}"
            );
            assert!(
                status.contains(&format!("?? {stray}")),
                "{stray} must still be untracked in the working tree: {status:?}"
            );
            assert_eq!(
                fs::read_to_string(root.join(stray)).expect("stray still on disk"),
                format!("not for history: {stray}\n")
            );
        }
    }

    /// Honest label: DISCRIMINATOR for the `git add -A` sweep. On the unfixed binary every
    /// stray lands in the materialize commit, and this fails on the first one.
    ///
    /// The second half is the CONTROL that keeps the fix honest: the cheap way to pass the
    /// first half is to stage nothing untracked at all, which would leave the change's own
    /// workspace, deliberately left uncommitted here, out of the commit that records it.
    #[test]
    fn check_commit_never_commits_an_unrelated_untracked_file() {
        let temp = TempDir::new().expect("temp project");
        let root = temp.path();
        git_project_fixture(root);
        let id = approved_change_fixture(root);
        // The delivery is a tracked edit.
        fs::write(root.join("README.md"), "# fixture\n\nthe delivery\n").unwrap();
        write_strays(root);

        run_checked_commit(root, Some(&id), false, false, OutputFormat::Json)
            .expect("check --commit");

        assert_strays_left_alone(root, &root.join(".git"));

        let history = paths_in_history(&root.join(".git"));
        for owned in [
            "state.json",
            "change.md",
            "verification.json",
            "approvals.json",
        ] {
            let path = format!(".specsync/changes/{id}/{owned}");
            assert!(
                history.contains(&path),
                "the change's own untracked workspace must still be committed: {path} missing from {history:?}"
            );
        }
        // `change new` wrote this ledger in the freshly adopted fixture. Left out, the change
        // would reach history without the origin anchor every later read of it checks.
        assert!(
            history.contains(&".specsync/workflow-v2-baseline.json".to_string()),
            "the lifecycle ledger `change new` wrote must be committed: {history:?}"
        );
        let committed_readme = std::process::Command::new("git")
            .args(["show", "HEAD:README.md"])
            .current_dir(root)
            .output()
            .expect("git show");
        assert_eq!(
            String::from_utf8_lossy(&committed_readme.stdout),
            "# fixture\n\nthe delivery\n",
            "the tracked delivery edit must be committed"
        );
        // The fixture has no `.specsync/.gitignore`, so the project lock is untracked too. It is
        // a runtime file: never committed, never reported, and allowed to stay behind.
        let runtime = change::lifecycle_commit_scope(root, &id)
            .expect("scope")
            .runtime;
        let leftovers: Vec<String> = worktree_status(root)
            .into_iter()
            .filter(|line| !STRAYS.iter().any(|stray| *line == format!("?? {stray}")))
            .filter(|line| !runtime.iter().any(|path| *line == format!("?? {path}")))
            .collect();
        assert!(
            leftovers.is_empty(),
            "only the strays and runtime files may remain outside history: {leftovers:?}"
        );
    }

    /// Honest label: DISCRIMINATOR for the archive commit `ship --push` makes, which went
    /// through the same `git add -A`. It asserts on what reached the remote, because a pushed
    /// commit is the one that cannot be taken back.
    ///
    /// CONTROL in the same test: the archive package still reaches the remote, and the active
    /// workspace it moved out of is gone from the pushed tree.
    #[test]
    fn ship_push_never_commits_an_unrelated_untracked_file() {
        let temp = TempDir::new().expect("temp project");
        let root = temp.path();
        let remote_parent = TempDir::new().expect("temp remote");
        let remote = remote_parent.path().join("origin.git");
        git_project_fixture(root);
        git_in(
            remote_parent.path(),
            &["init", "--bare", "-b", "main", "origin.git"],
        );
        git_in(
            root,
            &["remote", "add", "origin", remote.to_str().expect("utf-8")],
        );
        git_in(root, &["push", "-u", "origin", "main"]);

        let id = approved_change_fixture(root);
        // Present from before verification, as in the reports: a stray written after review
        // would stale the review and block ship for an unrelated reason.
        write_strays(root);
        run_checked_commit(root, Some(&id), false, false, OutputFormat::Json)
            .expect("check --commit");
        change::record_scoped_review_with_verdict(
            root,
            &id,
            "Independent".into(),
            change::ScopedReviewVerdict::Pass,
        )
        .expect("review");

        run_ship(
            root,
            Some(&id),
            false,
            true,
            false,
            1,
            OutputFormat::Json,
            true,
        )
        .expect("ship --push");

        assert_strays_left_alone(root, &remote);

        let pushed = std::process::Command::new("git")
            .arg(format!("--git-dir={}", remote.display()))
            .args(["ls-tree", "-r", "--name-only", "main"])
            .output()
            .expect("git ls-tree");
        let pushed = String::from_utf8_lossy(&pushed.stdout).into_owned();
        assert!(
            pushed
                .lines()
                .any(|path| path.starts_with(".specsync/archive/changes/")
                    && path.ends_with(&format!("-{id}/finalization.json"))),
            "the archive package must reach the remote: {pushed}"
        );
        assert!(
            !pushed
                .lines()
                .any(|path| path.starts_with(&format!(".specsync/changes/{id}/"))),
            "the archived workspace must be gone from the pushed tree: {pushed}"
        );
        let subject = std::process::Command::new("git")
            .arg(format!("--git-dir={}", remote.display()))
            .args(["log", "-1", "--pretty=%s", "main"])
            .output()
            .expect("git log");
        assert_eq!(
            String::from_utf8_lossy(&subject.stdout).trim(),
            format!("chore(lifecycle): archive {id}")
        );
    }

    // Verifies REQ-cmd-change-022.
    #[test]
    fn finalize_commits_the_archive_move() {
        let temp = TempDir::new().expect("temp project");
        let root = temp.path();
        git_project_fixture(root);
        let id = approved_change_fixture(root);
        run_checked_commit(root, Some(&id), false, false, OutputFormat::Json)
            .expect("check --commit");
        change::record_scoped_review_with_verdict(
            root,
            &id,
            "Independent".into(),
            change::ScopedReviewVerdict::Pass,
        )
        .expect("review");

        cmd_change(
            root,
            ChangeAction::Finalize { id: id.clone() },
            OutputFormat::Text,
            false,
        );

        let message = std::process::Command::new("git")
            .args(["log", "-1", "--format=%s"])
            .current_dir(root)
            .output()
            .expect("log");
        let message = String::from_utf8_lossy(&message.stdout);
        assert!(
            message.contains(&format!("chore(lifecycle): archive {id}")),
            "{message}"
        );
        let status = std::process::Command::new("git")
            .args(["status", "--porcelain"])
            .current_dir(root)
            .output()
            .expect("status");
        let status = String::from_utf8_lossy(&status.stdout);
        assert!(
            !status.contains(&format!(".specsync/changes/{id}")),
            "the archive move must be committed: {status}"
        );
    }

    // Verifies REQ-cmd-change-022. The move can land before the commit does.
    #[test]
    fn finalize_commits_an_archive_that_was_left_uncommitted() {
        let temp = TempDir::new().expect("temp project");
        let root = temp.path();
        git_project_fixture(root);
        let id = approved_change_fixture(root);
        run_checked_commit(root, Some(&id), false, false, OutputFormat::Json)
            .expect("check --commit");
        change::record_scoped_review_with_verdict(
            root,
            &id,
            "Independent".into(),
            change::ScopedReviewVerdict::Pass,
        )
        .expect("review");
        change::finalize_change(root, &id).expect("archive move");

        cmd_change(
            root,
            ChangeAction::Finalize { id: id.clone() },
            OutputFormat::Text,
            false,
        );

        let message = std::process::Command::new("git")
            .args(["log", "-1", "--format=%s"])
            .current_dir(root)
            .output()
            .expect("log");
        let message = String::from_utf8_lossy(&message.stdout);
        assert!(
            message.contains(&format!("chore(lifecycle): archive {id}")),
            "{message}"
        );
        let printed = std::process::Command::new("git")
            .args(["status", "--porcelain"])
            .current_dir(root)
            .output()
            .expect("status");
        let printed = String::from_utf8_lossy(&printed.stdout);
        assert!(
            !printed.contains(&format!(".specsync/changes/{id}")),
            "{printed}"
        );
    }

    /// Porcelain `-z` writes a rename as two fields, destination then source. Reading the source
    /// as an entry of its own would take the first two bytes of a filename as status codes. A
    /// staged rename, a tracked deletion, a tracked edit and a stray are each classified once:
    /// the edit and the deletion are staged, the stray is reported and left untracked.
    #[test]
    fn staging_reads_each_porcelain_entry_once_and_stages_only_tracked_edits() {
        let temp = TempDir::new().expect("temp project");
        let root = temp.path();
        git_in(root, &["init", "-b", "main"]);
        git_in(root, &["config", "user.email", "t@example.com"]);
        git_in(root, &["config", "user.name", "T"]);
        for name in ["moved.txt", "gone.txt", "edited.txt"] {
            fs::write(root.join(name), format!("{name}\n")).unwrap();
        }
        git_in(root, &["add", "."]);
        git_in(root, &["commit", "-m", "base"]);
        git_in(root, &["mv", "moved.txt", "renamed.txt"]);
        fs::remove_file(root.join("gone.txt")).unwrap();
        fs::write(root.join("edited.txt"), "edited\n").unwrap();
        fs::write(root.join("exp2.sh"), "echo stray\n").unwrap();

        let scope = change::LifecycleCommitScope {
            owned: Vec::new(),
            runtime: Vec::new(),
        };
        let left_out = stage_lifecycle_paths(root, &scope, true).expect("stage");

        assert_eq!(left_out, vec!["exp2.sh".to_string()]);
        let staged = worktree_status(root);
        for expected in [
            "R  moved.txt -> renamed.txt",
            "D  gone.txt",
            "M  edited.txt",
            "?? exp2.sh",
        ] {
            assert!(
                staged.contains(&expected.to_string()),
                "missing `{expected}`: {staged:?}"
            );
        }
    }

    /// A scope owns itself and what lies beneath it, never a sibling sharing its prefix: the
    /// workspace of `add-auth` must not own the workspace of `add-auth-v2`.
    #[test]
    fn a_lifecycle_scope_owns_its_subtree_and_not_a_prefix_sibling() {
        let workspace = ".specsync/changes/add-auth";
        assert!(path_is_within(workspace, workspace));
        assert!(path_is_within(
            ".specsync/changes/add-auth/state.json",
            workspace
        ));
        assert!(path_is_within(
            ".specsync/changes/add-auth/deltas/auth.md",
            &format!("{workspace}/")
        ));
        assert!(!path_is_within(
            ".specsync/changes/add-auth-v2/state.json",
            workspace
        ));
        assert!(!path_is_within(".specsync/changes", workspace));
        assert!(!path_is_within("exp2.sh", workspace));
    }

    /// The warning names each left-out file, bounds a long list, and tells the author what
    /// removing one does to verification only when there is verification to stale.
    #[test]
    fn the_left_out_warning_names_the_files_and_what_to_do() {
        let few: Vec<String> = STRAYS.iter().map(|stray| stray.to_string()).collect();
        let few_refs: Vec<&String> = few.iter().collect();
        let checked = left_out_warning(&few_refs, Some("add-auth"));
        for stray in STRAYS {
            assert!(checked.contains(&format!("\n  {stray}")), "{checked}");
        }
        assert!(checked.contains("git add"), "{checked}");
        assert!(
            checked.contains("specsync change check add-auth --commit"),
            "{checked}"
        );
        let archived = left_out_warning(&few_refs, None);
        assert!(!archived.contains("change check"), "{archived}");

        let many: Vec<String> = (0..LEFT_OUT_LISTING_LIMIT + 5)
            .map(|index| format!("scratch/{index}.log"))
            .collect();
        let many_refs: Vec<&String> = many.iter().collect();
        let bounded = left_out_warning(&many_refs, None);
        assert!(bounded.contains("… and 5 more"), "{bounded}");
        assert!(!bounded.contains(&format!("scratch/{}.log", LEFT_OUT_LISTING_LIMIT)));

        // A second commit in the same run does not repeat what the first already named.
        let mut disclosed = few.clone();
        let before = disclosed.len();
        disclose_left_out(&few, &mut disclosed, None);
        assert_eq!(disclosed.len(), before);
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
    /// Verifies REQ-cmd-change-031 (Codex review of #903). DISCRIMINATOR: a workflow-v2 change
    /// accepted by a finalization that stopped before the archive move, with a task reopened
    /// before the retry status points at. ship-status lists the task as a blocker and names it
    /// first, the text status names it, and ship refuses naming the task and its own command.
    #[test]
    fn ship_names_a_task_reopened_after_acceptance() {
        let temp = TempDir::new().expect("temp project");
        let root = temp.path();
        git_project_fixture(root);
        let record = change::create_change(
            root,
            CreateChangeRequest {
                description: "Ship a task reopened after acceptance".into(),
                kind: ChangeKind::Documentation,
                affected_specs: Vec::new(),
                affected_paths: vec!["README.md".into()],
                requested_artifacts: vec![ArtifactKind::Tasks],
                no_spec_change: true,
                rationale: Some("Command test fixture".into()),
            },
        )
        .expect("create draft");
        let id = record.id.clone();
        approve_and_implement_fixture(root, &id);
        git_in(root, &["add", "."]);
        git_in(root, &["commit", "-m", "implement"]);
        change::check_change(root, Some(&id)).expect("check");
        git_in(root, &["add", "."]);
        git_in(root, &["commit", "-m", "record verification"]);
        // Acceptance binds verification to HEAD, so verify the committed tree once more.
        change::check_change(root, Some(&id)).expect("check the committed tree");
        // A finalization that stopped here: accepted, still in its workspace.
        let accepted = change::accept_change(root, &id, None, None).expect("accept");
        assert_eq!(accepted.state, ChangeState::Accepted);
        assert!(accepted.workflow_version >= 2, "the fixture is workflow v2");
        let tasks = root.join(".specsync/changes").join(&id).join("tasks.md");
        fs::write(&tasks, "# Tasks\n\n- [ ] Reopened after acceptance.\n").expect("reopen");

        let record = change::load_change(root, &id).expect("reload");
        let report = ship_status_report(root, &record).expect("ship status");
        assert_eq!(report["ready_to_finalize"], false, "{report}");
        let blockers = report["blockers"].as_array().expect("blockers");
        assert!(
            blockers
                .iter()
                .any(|blocker| blocker.as_str().is_some_and(|text| text
                    .starts_with("tasks.md contains incomplete tasks: finish the 1 open task"))),
            "ship-status lists the reopened task: {blockers:?}"
        );
        let next = report["ship_next"].as_str().expect("ship_next");
        assert!(next.starts_with("finish the 1 open task"), "{next}");
        let text = text_mode_next_action(root, &record);
        assert!(text.starts_with("finish the 1 open task"), "{text}");

        let refusal = run_ship(
            root,
            Some(&id),
            false,
            false,
            false,
            900,
            OutputFormat::Json,
            false,
        )
        .expect_err("ship refuses the open task");
        assert!(
            refusal.starts_with("tasks.md contains incomplete tasks"),
            "{refusal}"
        );
        assert!(
            refusal.ends_with(&format!("then re-run `specsync change ship {id}`")),
            "{refusal}"
        );
    }

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
    /// REQ-cmd-change-007: archive-first CI runs nothing on a tip with an open change, so no
    /// guidance may send the operator to push a product tip and wait, or to reuse a parent's
    /// trust. Stage ids stay stable for JSON consumers.
    #[test]
    fn ship_guidance_teaches_archive_first_ci() {
        assert!(SHIP_TRUST_LOCAL_GUIDANCE.starts_with("Archive-first CI"));
        for stale in [
            "reuse trust",
            "push the product tip",
            "review_only or archive_only tip",
            "implementation ready",
        ] {
            assert!(
                !SHIP_TRUST_LOCAL_GUIDANCE.contains(stale),
                "local guidance still teaches the product-tip flow: {stale}"
            );
        }

        let temp = TempDir::new().expect("temp project");
        let root = temp.path();
        git_project_fixture(root);
        git_in(root, &["switch", "-c", "feature"]);
        let id = reviewed_change_fixture(root);
        let record = change::load_change(root, &id).expect("reload");
        let report = ship_status_report(root, &record).expect("ship status");

        let stages = report["stages"].as_array().expect("stages");
        let ids: Vec<&str> = stages
            .iter()
            .filter_map(|stage| stage["id"].as_str())
            .collect();
        assert_eq!(ids, ["product_tip", "review_tip", "archive_tip", "merge"]);
        assert_eq!(stages[0]["title"], "Product tip (local verification)");
        for stage in stages {
            let text = format!("{} {}", stage["title"], stage["action"]);
            for stale in [
                "full CI)",
                "wait for trust",
                "review_only tip",
                "trust reuse",
            ] {
                assert!(
                    !text.contains(stale),
                    "stage still teaches the product-tip flow: {text}"
                );
            }
        }
    }

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

        assert_eq!(report["stages"][0]["id"], "product_tip");
        assert_eq!(
            report["stages"][0]["status"], "done",
            "current content must complete the product stage despite rewritten ancestry: {report}"
        );
        assert_eq!(
            report["stages"][0]["action"],
            "product verification evidence is current for this content"
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

    #[test]
    fn ship_status_product_stage_rejects_stale_content_with_ancestor_evidence() {
        let temp = TempDir::new().expect("temp project");
        let root = temp.path();
        git_project_fixture(root);
        let id = reviewed_change_fixture(root);
        fs::write(
            root.join("README.md"),
            "# fixture\n\nchanged after verification\n",
        )
        .unwrap();
        git_in(root, &["add", "."]);
        git_in(root, &["commit", "-m", "change verified content"]);

        let record = change::load_change(root, &id).expect("reload");
        let report = ship_status_report(root, &record).expect("ship status");
        assert_eq!(report["verification_ancestor_of_head"], true);
        assert!(!change::recorded_verification_is_current(root, &record));
        assert_eq!(report["ready_to_finalize"], false);
        assert_eq!(report["stages"][0]["status"], "current", "{report}");
    }

    #[test]
    fn ship_status_product_stage_rejects_missing_verification() {
        let temp = TempDir::new().expect("temp project");
        let root = temp.path();
        git_project_fixture(root);
        let id = reviewed_change_fixture(root);
        fs::remove_file(
            root.join(".specsync/changes")
                .join(&id)
                .join("verification.json"),
        )
        .unwrap();
        let record = change::load_change(root, &id).expect("reload");
        let report = ship_status_report(root, &record).expect("ship status");
        assert_eq!(report["ready_to_finalize"], false);
        assert_eq!(report["stages"][0]["status"], "current", "{report}");
    }

    #[test]
    fn ship_status_treats_retired_as_terminal_in_every_stage() {
        let temp = TempDir::new().expect("temp project");
        let root = temp.path();
        git_project_fixture(root);
        let record = change::create_change(
            root,
            CreateChangeRequest {
                description: "Retire a terminal record's ship status".into(),
                kind: ChangeKind::Documentation,
                affected_specs: Vec::new(),
                affected_paths: vec!["README.md".into()],
                requested_artifacts: Vec::new(),
                no_spec_change: true,
                rationale: Some("Command test fixture".into()),
            },
        )
        .expect("create draft");
        let id = record.id.clone();
        // A retired record without its receipt is the invalid-custody shape; the stage
        // semantics under test (terminal, no lane guidance) hold either way.
        let mut retired = change::load_change(root, &id).expect("load");
        retired.state = ChangeState::Retired;
        let state_path = root.join(".specsync/changes").join(&id).join("state.json");
        fs::write(
            &state_path,
            serde_json::to_string_pretty(&retired).expect("serialize"),
        )
        .expect("write retired state");

        let record = change::load_change(root, &id).expect("reload");
        let report = ship_status_report(root, &record).expect("ship status");
        let stages = report["stages"].as_array().expect("stages");
        assert!(
            stages.iter().all(|stage| stage["status"] == "done"),
            "every stage is terminal for a retired record: {stages:?}"
        );
        assert_eq!(report["ready_to_finalize"], false);
        let mut rendered = String::new();
        for stage in stages {
            rendered.push_str(stage["action"].as_str().unwrap_or_default());
        }
        for warning in report["warnings"].as_array().expect("warnings") {
            rendered.push_str(warning.as_str().unwrap_or_default());
        }
        rendered.push_str(report["ship_next"].as_str().unwrap_or_default());
        for guidance in [
            "change review",
            "change ship",
            "change check",
            "merge only after finalize",
            "still active",
        ] {
            assert!(
                !rendered.contains(guidance),
                "a retired record must not be guided through the shipping lane: `{guidance}` in {rendered}"
            );
        }
    }

    /// DISCRIMINATOR (issue #825). An invalid `verification_ignored_paths` entry is a blocker
    /// that names the entry and the rule, not "stale verification": ship-status reads currency
    /// through a boolean helper that would otherwise hide the refusal. The list is written as raw
    /// JSON so this test also runs against a binary that does not know the key.
    /// Verifies REQ-cmd-change-024.
    #[test]
    fn ship_status_blocks_on_an_invalid_verification_ignored_paths_entry() {
        let temp = TempDir::new().expect("temp project");
        let root = temp.path();
        git_project_fixture(root);
        let id = reviewed_change_fixture(root);
        let policy_path = root.join(".specsync/sdd.json");
        let mut policy: serde_json::Value =
            serde_json::from_slice(&fs::read(&policy_path).unwrap()).unwrap();
        policy["verification_ignored_paths"] = serde_json::json!(["src/"]);
        fs::write(&policy_path, serde_json::to_vec_pretty(&policy).unwrap()).unwrap();

        let record = change::load_change(root, &id).expect("reload");
        let report = ship_status_report(root, &record).expect("ship status");
        let blockers: Vec<&str> = report["blockers"]
            .as_array()
            .map(|values| values.iter().filter_map(|value| value.as_str()).collect())
            .unwrap_or_default();
        assert!(
            blockers.iter().any(|blocker| {
                blocker.contains("verification_ignored_paths")
                    && blocker.contains("`src/`")
                    && blocker.contains("source_dirs")
            }),
            "{blockers:?}"
        );
        assert!(
            !blockers.iter().any(|blocker| blocker.contains("stale")),
            "the entry is the blocker, not stale verification: {blockers:?}"
        );
        assert_eq!(report["ready_to_finalize"], false);
    }

    /// DISCRIMINATOR for the non-empty case, CONTROL for the empty one (NEW API: the helper does
    /// not exist on a80748ff; the integration test asserts the same lines through the binary).
    /// Verifies REQ-cmd-change-024.
    #[test]
    fn status_and_check_text_render_the_list_only_when_non_empty() {
        assert!(verification_ignored_lines(&[]).is_empty());
        assert_eq!(
            verification_ignored_lines(&[".chough/".to_string(), ".tool/".to_string()]),
            vec![
                "  Verification ignores: .chough/, .tool/".to_string(),
                format!("    note: {VERIFICATION_IGNORED_NOTE}"),
            ]
        );
        assert!(VERIFICATION_IGNORED_NOTE.contains("cannot tell whether project tests or CI"));
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
/// Nothing is committed unless the first verification passes. The second pass re-verifies the
/// same content against the commit the first one made. If it fails anyway, that materialize
/// commit stays on the branch and is not rewound, and the error names it along with the command
/// that resumes. Moving a branch tip back on the author's behalf is worse than leaving a commit
/// they can see and finish.
///
/// Each commit stages the change's own paths and the project's tracked edits, never other
/// untracked files; see [`git_commit_lifecycle`].
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

    // A refusal names this invocation, flags and all, as the command to re-run (REQ-change-131).
    let invocation = change::CheckInvocation {
        strict,
        commit: true,
        push,
    };
    let verify = |label: &str| -> Result<Option<String>, String> {
        say(label);
        let record = change::check_change_as(root, id, invocation)?;
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

    let mut disclosed = Vec::new();
    let materialized = git_commit_lifecycle(
        root,
        &change::lifecycle_commit_scope(root, &resolved)?,
        &format!("chore(lifecycle): materialize {resolved}"),
        true,
    )?;
    disclose_left_out(&materialized.left_out, &mut disclosed, Some(&resolved));
    let materialize_commit = if materialized.committed {
        git_rev_parse(root, "HEAD").ok()
    } else {
        None
    };
    say(if materialized.committed {
        "Committed the materialized spec."
    } else {
        "Nothing new to commit for the materialized spec."
    });

    // Second pass: re-anchor against the committed tree. Evidence files live under
    // `.specsync/changes/`, which is excluded from the project-input digest, so
    // committing them cannot stale this result.
    if let Err(error) = verify("Checking (2/2): re-verifying against the committed tree…") {
        if !materialized.committed {
            return Err(error);
        }
        let head = git_rev_parse(root, "HEAD").unwrap_or_else(|_| "HEAD".into());
        return Err(format!(
            "{error}\nnote: the first pass already committed `chore(lifecycle): materialize {resolved}` as {head:.8} and it was left in place; fix the failure, then re-run `specsync change check {resolved} --commit`"
        ));
    }
    let recorded = git_commit_lifecycle(
        root,
        &change::lifecycle_commit_scope(root, &resolved)?,
        &format!("chore(lifecycle): record {resolved} verification"),
        true,
    )?;
    disclose_left_out(&recorded.left_out, &mut disclosed, Some(&resolved));
    let verification_commit = if recorded.committed {
        git_rev_parse(root, "HEAD").ok()
    } else {
        None
    };
    say("Committed the verification evidence.");

    if push {
        run_git(root, &["push"])?;
        say("Pushed.");
    }
    say("Verified, committed, and consistent with the committed tree.");
    if quiet {
        // #839: a machine caller asked for JSON; the commit path must answer with one
        // document, not silence (REQ-change-004).
        println!(
            "{}",
            serde_json::json!({
                "change": resolved,
                "verified": true,
                "materialize_commit": materialize_commit,
                "verification_commit": verification_commit,
                "pushed": push,
                "left_out": disclosed,
                // REQ-cmd-change-unrun-commands-001 (t384's option a for #839): the outcome
                // document carries the same disclosure every other JSON success output does.
                "configured_commands_not_run": configured_commands_not_run(root)?,
            })
        );
        return Ok(());
    }
    print_configured_commands_not_run(root)?;
    print_verification_ignored_lines(root);
    Ok(())
}

fn run_git(root: &std::path::Path, args: &[&str]) -> Result<(), String> {
    let output = crate::git_utils::output_after_child_exits(
        std::process::Command::new("git")
            .args(args)
            .current_dir(root),
    );
    if !crate::git_utils::is_read_only_git_invocation(args) {
        crate::git_utils::invalidate_repository_facts();
    }
    let output =
        output.map_err(|error| format!("failed to run git {}: {error}", args.join(" ")))?;
    if !output.status.success() {
        return Err(format!(
            "git {} failed: {}",
            args.join(" "),
            String::from_utf8_lossy(&output.stderr).trim()
        ));
    }
    Ok(())
}

/// What one lifecycle commit did.
struct LifecycleCommit {
    /// False when nothing was staged, so no commit was made.
    committed: bool,
    /// Untracked files deliberately left out, as project-relative paths.
    left_out: Vec<String>,
}

/// Stage what a lifecycle commit owns and commit it, treating "nothing to commit" as success so
/// the sequence stays idempotent when a pass produced no change.
///
/// `scope` is [`change::lifecycle_commit_scope`] for the change being committed. This used to be
/// `git add -A`, which put every untracked, non-ignored file in the project into a commit that
/// `--push` then published. See [`stage_lifecycle_paths`] for what is staged now.
fn git_commit_lifecycle(
    root: &Path,
    scope: &change::LifecycleCommitScope,
    message: &str,
    include_tracked_edits: bool,
) -> Result<LifecycleCommit, String> {
    // Every lifecycle commit can commit a sequence ledger that went stale while
    // the branch sat. Flooring here rather than at the single call site named in
    // the report covers all three, which is the difference between fixing this
    // and fixing where it was noticed.
    // Reported on stderr rather than through the caller's `say`: this is a state
    // correction, not progress chatter, so it must survive `--quiet`, and it must
    // stay off stdout where a `--format json` payload is being written.
    if let Some((was, now)) = change::floor_sequence_ledger_to_committed(root)? {
        eprintln!(
            "note: raised the change sequence ledger from {was} to the committed {now} before staging; \
             a ledger written before the branch caught up would have committed a lower high-water mark"
        );
    }
    let left_out = stage_lifecycle_paths(root, scope, include_tracked_edits)?;
    let status = crate::git_utils::status_without_inherited_output(
        std::process::Command::new("git")
            .args(["diff", "--cached", "--quiet"])
            .current_dir(root),
    )
    .map_err(|error| format!("failed to inspect staged changes: {error}"))?;
    if status.success() {
        return Ok(LifecycleCommit {
            committed: false,
            left_out,
        });
    }
    run_git(root, &["commit", "-m", message])?;
    Ok(LifecycleCommit {
        committed: true,
        left_out,
    })
}

/// How many paths go to one `git add`, well inside every platform's argument limit.
const STAGE_BATCH_PATHS: usize = 200;

/// Stage the project's tracked edits and the untracked files under `scope.owned`, by explicit
/// literal pathspecs. Returns every other untracked file, which is left unstaged, except the
/// lifecycle's own runtime files, which are neither staged nor returned.
///
/// A tracked edit is part of the delivery. Verification digests the working tree, so a tracked
/// edit left unstaged would be verified and never committed. An untracked file is different: it
/// may be a private debug archive, an agent's scratch directory or an experiment script, and
/// nothing about it being on disk says it belongs in history. Of those, only the ones the
/// lifecycle wrote or the change owns are staged. The rest are the author's to stage.
///
/// The index is not reset. Whatever the author already staged is committed with the rest, as it
/// always was. Unmerged entries are not staged, so an unresolved conflict still stops the commit
/// instead of being recorded as resolved.
fn stage_lifecycle_paths(
    root: &Path,
    scope: &change::LifecycleCommitScope,
    include_tracked_edits: bool,
) -> Result<Vec<String>, String> {
    // Porcelain paths are relative to the repository top-level, and `git add` resolves them
    // against the working directory. Both are brought to project-relative form so that a project
    // nested inside a larger repository stages its own paths and only those.
    let prefix = git_output_text(root, &["rev-parse", "--show-prefix"])?
        .trim_end_matches(['\n', '\r'])
        .to_string();
    let output = crate::git_utils::output_after_child_exits(
        std::process::Command::new("git")
            .args([
                "status",
                "--porcelain=v1",
                "-z",
                "--untracked-files=all",
                "--",
                ".",
            ])
            .current_dir(root),
    )
    .map_err(|error| format!("failed to run git status: {error}"))?;
    if !output.status.success() {
        return Err(format!(
            "git status failed: {}",
            String::from_utf8_lossy(&output.stderr).trim()
        ));
    }

    let mut stage = Vec::new();
    let mut left_out = Vec::new();
    let mut fields = output.stdout.split(|byte| *byte == 0);
    while let Some(entry) = fields.next() {
        if entry.len() < 4 {
            continue;
        }
        let (index, worktree) = (entry[0], entry[1]);
        // A rename or copy, in either column, carries its source as the next field. The source
        // is already recorded as removed, so only the destination is acted on.
        if matches!(index, b'R' | b'C') || matches!(worktree, b'R' | b'C') {
            fields.next();
        }
        let path = String::from_utf8_lossy(&entry[3..]).into_owned();
        let Some(path) = path.strip_prefix(prefix.as_str()).map(str::to_owned) else {
            continue;
        };
        let unmerged = index == b'U'
            || worktree == b'U'
            || (index == b'A' && worktree == b'A')
            || (index == b'D' && worktree == b'D');
        if unmerged {
            continue;
        }
        if index == b'?' {
            if scope.owned.iter().any(|owned| path_is_within(&path, owned)) {
                stage.push(path);
            } else if !scope
                .runtime
                .iter()
                .any(|runtime| path_is_within(&path, runtime))
            {
                left_out.push(path);
            }
        } else if worktree != b' '
            && (include_tracked_edits
                || scope.owned.iter().any(|owned| path_is_within(&path, owned)))
        {
            stage.push(path);
        }
    }

    for batch in stage.chunks(STAGE_BATCH_PATHS) {
        let mut args = vec!["--literal-pathspecs", "add", "--"];
        args.extend(batch.iter().map(String::as_str));
        run_git(root, &args)?;
    }
    left_out.sort();
    Ok(left_out)
}

/// True when `path` is `scope` itself or lies beneath it.
fn path_is_within(path: &str, scope: &str) -> bool {
    let scope = scope.trim_end_matches('/');
    path == scope
        || path
            .strip_prefix(scope)
            .is_some_and(|rest| rest.starts_with('/'))
}

fn git_output_text(root: &Path, args: &[&str]) -> Result<String, String> {
    let output = crate::git_utils::output_after_child_exits(
        std::process::Command::new("git")
            .args(args)
            .current_dir(root),
    )
    .map_err(|error| format!("failed to run git {}: {error}", args.join(" ")))?;
    if !output.status.success() {
        return Err(format!(
            "git {} failed: {}",
            args.join(" "),
            String::from_utf8_lossy(&output.stderr).trim()
        ));
    }
    Ok(String::from_utf8_lossy(&output.stdout).into_owned())
}

/// How many left-out paths a warning names before it summarizes the rest.
const LEFT_OUT_LISTING_LIMIT: usize = 20;

/// Name, on stderr, the untracked files a lifecycle commit left out. `disclosed` carries what an
/// earlier commit of the same run already named, so the second commit does not repeat it.
///
/// Stderr, like the ledger note: it must survive `--quiet` and keep `--format json` stdout a
/// single document. `verified_change` is set when the files sit under recorded verification, which
/// digests the working tree including them.
fn disclose_left_out(
    left_out: &[String],
    disclosed: &mut Vec<String>,
    verified_change: Option<&str>,
) {
    let fresh: Vec<&String> = left_out
        .iter()
        .filter(|path| !disclosed.contains(path))
        .collect();
    if fresh.is_empty() {
        return;
    }
    eprintln!("{}", left_out_warning(&fresh, verified_change));
    disclosed.extend(fresh.into_iter().cloned());
}

fn left_out_warning(paths: &[&String], verified_change: Option<&str>) -> String {
    let mut out = format!(
        "warning: left {} untracked file(s) out of this lifecycle commit. It commits only the change's \
         own workspace, its specs, the lifecycle ledgers and tracked edits:",
        paths.len()
    );
    for path in paths.iter().take(LEFT_OUT_LISTING_LIMIT) {
        out.push_str(&format!("\n  {path}"));
    }
    if paths.len() > LEFT_OUT_LISTING_LIMIT {
        out.push_str(&format!(
            "\n  … and {} more",
            paths.len() - LEFT_OUT_LISTING_LIMIT
        ));
    }
    out.push_str(
        "\n  `git add` and commit any that belong to this delivery. Move, delete, or .gitignore the rest.",
    );
    if let Some(id) = verified_change {
        out.push_str(&format!(
            "\n  The recorded verification digests them where they sit. Committing one keeps it current; \
             removing one stales it, so re-run `specsync change check {id} --commit` afterwards."
        ));
    }
    out
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

        assert_eq!(next, "Merge the pull request when the checks are green.");
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
                "Merge the pull request when the checks are green.",
            ),
            (
                true,
                false,
                "Wait until the checks are green, then merge the pull request.",
            ),
            (
                false,
                true,
                "Push this archive, wait until the checks are green, then merge the pull request.",
            ),
            (
                false,
                false,
                "Push this archive, wait until the checks are green, then merge the pull request.",
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
        assert_eq!(
            next,
            "Archive `other-change` next. This pull request is not finished."
        );
        assert!(!next.to_ascii_lowercase().contains("merge"), "{next}");
    }

    #[test]
    fn finalize_text_names_the_lesson_bundle_beside_the_merge_instruction() {
        let bundle = lesson_bundle_line();
        let next = ship_next_action(
            true,
            true,
            &[],
            &["specs/change/context.md".to_string()],
            "/archive/lesson-bundle.md",
        );
        assert!(bundle.starts_with("Lesson bundle: "), "{bundle}");
        assert!(bundle.contains("lesson-bundle.md"), "{bundle}");
        assert!(!bundle.contains("Next:"), "{bundle}");
        assert_eq!(next, "Merge the pull request when the checks are green.");
        assert!(!next.contains("write lessons"), "{next}");
        assert!(!next.contains("Lesson bundle"), "{next}");
    }

    #[test]
    fn ship_commits_the_archive_and_names_merge_as_what_is_left() {
        let next = ship_next_action(false, false, &[], &[], "lesson-bundle.md");
        assert_eq!(
            next,
            "Push this archive, wait until the checks are green, then merge the pull request."
        );
        assert!(!pull_request_archive_sentence(&[]).contains("not finished"));
        assert_eq!(
            pull_request_archive_sentence(&[]),
            "All changes are archived. Ready to merge when the other checks are green."
        );
    }

    #[test]
    fn change_pr_next_step_waits_for_green_checks_and_does_not_merge() {
        let next = pull_request_ready_next();
        assert_eq!(
            next,
            "Wait until the checks are green, then merge the pull request."
        );
        assert!(!next.contains("gh pr merge"), "{next}");
    }

    #[test]
    fn status_says_which_open_change_to_archive_first() {
        let open = vec![
            ArchiveCandidate {
                id: "draft-notes".into(),
                state: "draft".into(),
                ship: false,
            },
            ArchiveCandidate {
                id: "zebra-feature".into(),
                state: "verifying".into(),
                ship: true,
            },
        ];
        let sentence = pull_request_archive_sentence(&open);
        assert!(
            sentence
                .starts_with("This pull request is not finished. Archive `zebra-feature` first."),
            "{sentence}"
        );
        assert!(sentence.contains("Run `specsync change ship zebra-feature`."));
        assert!(!sentence.contains("Ready to merge"), "{sentence}");
    }

    #[test]
    fn accepted_is_named_before_a_verifying_change() {
        let open = vec![
            ArchiveCandidate {
                id: "later-verify".into(),
                state: "verifying".into(),
                ship: true,
            },
            ArchiveCandidate {
                id: "sooner-accept".into(),
                state: "accepted".into(),
                ship: false,
            },
        ];
        let sentence = pull_request_archive_sentence(&open);
        assert!(
            sentence.contains("Archive `sooner-accept` first"),
            "{sentence}"
        );
        assert!(
            sentence.contains("Run `specsync change status sooner-accept`."),
            "{sentence}"
        );
        assert_eq!(
            archive_sentence_without_command(&sentence),
            "This pull request is not finished. Archive `sooner-accept` first. Still open: `sooner-accept`, `later-verify`."
        );
    }
}

#[cfg(test)]
mod ship_command_tests {
    use super::*;
    use crate::cli::{Cli, Command};
    use clap::Parser;

    /// The command an open-task refusal names leaves the timeout out exactly when it is the
    /// default, so this constant must be the CLI's default.
    #[test]
    fn ship_wait_timeout_default_matches_the_cli() {
        let parsed =
            Cli::try_parse_from(["specsync", "change", "ship", "guide", "--wait"]).unwrap();
        match parsed.command {
            Some(Command::Change {
                action:
                    ChangeAction::Ship {
                        wait_timeout_secs, ..
                    },
            }) => assert_eq!(wait_timeout_secs, SHIP_WAIT_TIMEOUT_DEFAULT_SECS),
            _ => panic!("`change ship` parses as a ship action"),
        }
    }

    /// Verifies REQ-cmd-change-031: the named ship command repeats the flags it was given.
    #[test]
    fn ship_command_repeats_its_flags() {
        assert_eq!(
            ship_command("guide", false, false, false, 900),
            "specsync change ship guide"
        );
        assert_eq!(
            ship_command("guide", true, false, false, 900),
            "specsync change ship guide --dry-run"
        );
        assert_eq!(
            ship_command("guide", false, true, true, 900),
            "specsync change ship guide --push --wait"
        );
        assert_eq!(
            ship_command("guide", false, true, true, 60),
            "specsync change ship guide --push --wait --wait-timeout-secs 60"
        );
        assert_eq!(
            ship_command("guide", false, false, false, 60),
            "specsync change ship guide",
            "the timeout only matters with --wait"
        );
        let words: Vec<String> = ship_command("guide", false, true, true, 60)
            .split(' ')
            .map(str::to_string)
            .collect();
        Cli::try_parse_from(&words).expect("the named command parses");
    }
}
