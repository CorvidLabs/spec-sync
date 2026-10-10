use super::archive_guidance::{
    open_archive_candidates, ordered_archive_id, pull_request_archive_sentence,
    pull_request_sentence, ship_next_action,
};
use super::lifecycle_commit::{disclose_left_out, git_commit_lifecycle, git_output_text, run_git};
use super::render::{lesson_bundle_line, print_handoff_line_for, print_json};
use super::repo_status::git_rev_parse;
use super::ship_status::{print_ship_status, ship_status_report};
use crate::change;
use crate::change::ChangeRecord;
use crate::change::ChangeState;
use crate::types::OutputFormat;
use colored::Colorize;
use std::path::Path;

/// Archive the change when it is still open, push the branch, and open or reuse the pull request.
///
/// Merging stays a separate act. A change that is not ready is refused with the one command
/// that would make it ready.
pub(super) fn run_prepare_pr(
    root: &Path,
    id: Option<&str>,
    format: OutputFormat,
) -> Result<(), String> {
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
        .status()
        .map_err(|error| {
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
    let created = command
        .output()
        .map_err(|error| format!("failed to run gh pr create: {error}"))?;
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

pub(super) fn pull_request_ready_next() -> &'static str {
    "Wait until the checks are green, then merge the pull request."
}

fn discovered_pr_base(root: &Path) -> Option<String> {
    git_output_text(root, &["rev-parse", "--abbrev-ref", "origin/HEAD"])
        .ok()
        .and_then(|value| value.trim().strip_prefix("origin/").map(str::to_string))
        .filter(|base| !base.is_empty())
}

#[allow(clippy::too_many_arguments)]
pub(super) fn run_ship(
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

    // Soft preflight: live trust is advisory unless --wait forces a green parent later.
    if announce && matches!(trust_status, "pending" | "failed" | "empty") {
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

    let open = open_archive_candidates(root)?;
    let before = pull_request_archive_sentence(&open);
    let first_id = ordered_archive_id(&open);
    if first_id.as_deref() != Some(record.id.as_str()) && !open.is_empty() {
        // One document. Printing here and returning Err made JSON mode emit two.
        return Err(before);
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
