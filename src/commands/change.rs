use colored::Colorize;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::{self, Command};

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
                    println!("  Ship sequence (do not skip waits):");
                    println!("    1. Product tip must already be green: trust + SpecSync trusted policy + SpecSync implementation ready");
                    println!("    2. Push this archive commit alone as the PR tip");
                    println!("    3. Wait for archive-integrity + Required CI gate");
                    println!("    4. Then merge the PR on GitHub");
                    println!("  See: specsync change ship-status");
                }
            }
            Ok(())
        }),
        ChangeAction::ShipStatus { id } => print_ship_status(root, id.as_deref(), format),
        ChangeAction::Check { id } => {
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
            // Lightweight only (no digest loaders): review.json presence + artifact files.
            let review_path = root
                .join(".specsync/changes")
                .join(&record.id)
                .join("review.json");
            if review_path.is_file() {
                format!(
                    "product tip green (trust+policy+implementation ready), then `specsync change finalize {id}`, push archive tip alone; see `specsync change ship-status`"
                )
            } else if change::artifacts_complete_for_guidance(root, record) {
                format!(
                    "run `specsync change review {id} --reviewer <other-than-approver> --verdict pass`, then ship-status"
                )
            } else {
                format!("run `specsync change check {id}` or complete verification")
            }
        }
        ChangeState::Accepted if record.workflow_version >= 2 => {
            format!(
                "run `specsync change finalize {id}` then push archive tip alone after product tip is green (`ship-status`)"
            )
        }
        ChangeState::Accepted => format!("run `specsync change archive {id}`"),
        ChangeState::Archived => "no further action".into(),
    }
}

fn print_ship_status(root: &Path, id: Option<&str>, format: OutputFormat) -> Result<(), String> {
    let report = ship_status_report(root, id)?;
    match format {
        OutputFormat::Json => {
            print_json(&report);
            Ok(())
        }
        _ => {
            println!("{}", "Ship status".bold());
            println!("  HEAD:   {}", report["head_sha"].as_str().unwrap_or(""));
            println!(
                "  Parent: {}",
                report["parent_sha"].as_str().unwrap_or("(root)")
            );
            println!(
                "  Tip class: {}",
                report["tip_class"].as_str().unwrap_or("unknown")
            );
            if let Some(change) = report.get("change") {
                if !change.is_null() {
                    println!(
                        "  Change: {} ({})",
                        change["id"].as_str().unwrap_or(""),
                        change["state"].as_str().unwrap_or("")
                    );
                    println!(
                        "  Review current: {}  Verification present: {}",
                        change["scoped_review_current"], change["verification_present"]
                    );
                }
            }
            println!("  Ship sequence:");
            if let Some(steps) = report["ship_sequence"].as_array() {
                for step in steps {
                    println!("    - {}", step.as_str().unwrap_or(""));
                }
            }
            println!("  Next: {}", report["next"].as_str().unwrap_or(""));
            Ok(())
        }
    }
}

fn ship_status_report(root: &Path, id: Option<&str>) -> Result<serde_json::Value, String> {
    let head = git_rev_parse(root, "HEAD")?;
    let parent = git_rev_parse(root, "HEAD^").ok();
    let tip_class = classify_head_tip(root, &head, parent.as_deref())?;
    let change_json = match resolve_ship_change(root, id)? {
        Some(record) => {
            let summary = change::summarize_change_with_strict(root, &record, false);
            let verification_present = root
                .join(".specsync/changes")
                .join(&record.id)
                .join("verification.json")
                .is_file()
                || root
                    .join(".specsync/archive/changes")
                    .read_dir()
                    .ok()
                    .into_iter()
                    .flatten()
                    .filter_map(|e| e.ok())
                    .any(|e| {
                        e.file_name().to_string_lossy().contains(
                            record
                                .id
                                .split('-')
                                .take(2)
                                .collect::<Vec<_>>()
                                .join("-")
                                .as_str(),
                        ) && e.path().join("verification.json").is_file()
                    });
            Some(serde_json::json!({
                "id": record.id,
                "state": record.state,
                "workflow_version": record.workflow_version,
                "scoped_review_current": summary.scoped_review_current,
                "artifacts_complete": summary.artifacts_complete,
                "approval_valid": summary.approval_valid,
                "verification_present": verification_present,
                "lifecycle_next": summary.next_action,
            }))
        }
        None => None,
    };

    let ship_sequence = vec![
        "Land product work and wait once for: trust + SpecSync trusted policy + SpecSync implementation ready on a product tip".to_string(),
        "Required CI gate may fail with 'run finalize' on product tips — expected until archive tip".to_string(),
        "Record scoped review; CI reuses trust/implementation from first-parent product ancestors (no tip dance)".to_string(),
        "Run `specsync change finalize <id>` and push the archive commit; archive-integrity reuses ancestor provenance".to_string(),
        "Wait for archive-integrity + Required CI gate, then merge".to_string(),
    ];

    let next = match (tip_class.as_str(), &change_json) {
        ("archive_only", _) => {
            "wait for archive-integrity + Required CI gate, then merge on GitHub".to_string()
        }
        ("review_only", Some(c)) if c["scoped_review_current"].as_bool() == Some(true) => {
            format!(
                "wait for trust reuse on this tip, then finalize {} and push archive tip alone",
                c["id"].as_str().unwrap_or("<id>")
            )
        }
        ("review_only", _) => "wait for trust reuse success on this review tip".to_string(),
        (_, Some(c))
            if c["state"] == "verifying" && c["scoped_review_current"].as_bool() != Some(true) =>
        {
            format!(
                "record scoped review for {}, keep product tip green, then finalize",
                c["id"].as_str().unwrap_or("<id>")
            )
        }
        (_, Some(c)) if c["state"] == "verifying" => {
            format!(
                "keep product tip green (trust+policy+implementation ready), then finalize {}",
                c["id"].as_str().unwrap_or("<id>")
            )
        }
        (_, Some(c)) if c["state"] == "implementing" || c["state"] == "approved" => {
            format!(
                "run `specsync change check {}`",
                c["id"].as_str().unwrap_or("<id>")
            )
        }
        _ => "follow ship_sequence; use --json for machine fields".to_string(),
    };

    Ok(serde_json::json!({
        "head_sha": head,
        "parent_sha": parent,
        "tip_class": tip_class,
        "change": change_json,
        "ship_sequence": ship_sequence,
        "next": next,
        "product_issues": ["#487", "#488", "#489"],
    }))
}

fn resolve_ship_change(
    root: &Path,
    id: Option<&str>,
) -> Result<Option<change::ChangeRecord>, String> {
    let _scope = change::begin_change_read_scope(root);
    if let Some(id) = id {
        return change::load_change(root, id).map(Some);
    }
    let active = change::list_changes(root);
    let delivering: Vec<_> = active
        .into_iter()
        .filter(|r| {
            matches!(
                r.state,
                ChangeState::Approved
                    | ChangeState::Implementing
                    | ChangeState::Verifying
                    | ChangeState::Accepted
            )
        })
        .collect();
    match delivering.len() {
        0 => Ok(None),
        1 => Ok(Some(delivering.into_iter().next().expect("len 1"))),
        _ => Err("multiple delivering changes; pass an explicit change id to ship-status".into()),
    }
}

fn git_rev_parse(root: &Path, rev: &str) -> Result<String, String> {
    let output = Command::new("git")
        .args(["-C", root.to_str().unwrap_or("."), "rev-parse", rev])
        .output()
        .map_err(|e| format!("git rev-parse failed: {e}"))?;
    if !output.status.success() {
        return Err(format!(
            "git rev-parse {rev} failed: {}",
            String::from_utf8_lossy(&output.stderr).trim()
        ));
    }
    Ok(String::from_utf8_lossy(&output.stdout).trim().to_string())
}

fn classify_head_tip(root: &Path, head: &str, parent: Option<&str>) -> Result<String, String> {
    let Some(parent) = parent else {
        return Ok("product".into());
    };
    let script = root.join(".github/scripts/classify-ci-paths.sh");
    if !script.is_file() {
        return Ok("unknown".into());
    }
    let diff = Command::new("git")
        .args([
            "-C",
            root.to_str().unwrap_or("."),
            "diff",
            "--name-status",
            "-z",
            "-M",
            parent,
            head,
        ])
        .output()
        .map_err(|e| format!("git diff failed: {e}"))?;
    if !diff.status.success() {
        return Ok("unknown".into());
    }
    let mut child = Command::new("bash")
        .arg(&script)
        .arg(root)
        .arg("false")
        .arg("name-status")
        .arg(parent)
        .stdin(std::process::Stdio::piped())
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped())
        .spawn()
        .map_err(|e| format!("classify-ci-paths failed to start: {e}"))?;
    use std::io::Write;
    if let Some(mut stdin) = child.stdin.take() {
        stdin
            .write_all(&diff.stdout)
            .map_err(|e| format!("classify stdin: {e}"))?;
    }
    let output = child
        .wait_with_output()
        .map_err(|e| format!("classify wait: {e}"))?;
    if !output.status.success() {
        return Ok("unknown".into());
    }
    let text = String::from_utf8_lossy(&output.stdout);
    let mut archive_only = false;
    let mut review_only = false;
    let mut full = false;
    for line in text.lines() {
        if let Some(v) = line.strip_prefix("archive_only=") {
            archive_only = v == "true";
        } else if let Some(v) = line.strip_prefix("review_only=") {
            review_only = v == "true";
        } else if let Some(v) = line.strip_prefix("full=") {
            full = v == "true";
        }
    }
    Ok(if archive_only {
        "archive_only".into()
    } else if review_only {
        "review_only".into()
    } else if full {
        "product".into()
    } else {
        "lifecycle_other".into()
    })
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
