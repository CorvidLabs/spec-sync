use super::render::{print_json, text_mode_next_action};
use super::repo_status::{HeadTip, classify_head_tip, git_commit_present, git_is_ancestor};
use crate::change;
use crate::change::ChangeRecord;
use crate::change::ChangeState;
use crate::types::OutputFormat;
use colored::Colorize;
use std::fs;
use std::path::Path;

/// Ship readiness for one change: HEAD tip class, verification tip health, review,
/// trust guidance for staged product → review → archive tips, and the merge-before-finalize trap.
///
/// When `GITHUB_TOKEN` is set and the git remote is a GitHub repo, queries check-runs for the
/// parent (or tip) SHA. Offline / no-token stays on `local_guidance`. Force offline with
/// `SPECSYNC_SHIP_LOCAL_GUIDANCE=1`.
pub(super) fn ship_status_report(
    root: &Path,
    record: &ChangeRecord,
) -> Result<serde_json::Value, String> {
    let questions = change::next_questions_for(root, record);
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

    let ship_next = if ready_to_finalize {
        format!("run `specsync change ship {}`", record.id)
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
pub(super) fn merge_before_finalize_warning(still_active: bool) -> String {
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
pub(super) fn multi_active_ordering_warnings(sibling_ids: &[String]) -> Vec<String> {
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
    verification_current: bool,
    review: ShipReviewStatus,
    ready_to_finalize: bool,
) -> Vec<serde_json::Value> {
    let id = record.id.as_str();
    let verified = verification_present && verification_current && verification_commit.is_some();
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
            "product verification evidence is current for this content".to_string()
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
                "re-record the scoped review: `specsync change review {id} --reviewer <human>` — the recorded review is `{}` against this tree, and finalize requires a current one",
                review.label()
            )
        } else if product_done {
            format!(
                "record scoped review: `specsync change review {id} --reviewer <human>` then push a review_only tip if CI requires it; wait for trust reuse"
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
            "change is archived; push it if you have not, then merge when the checks are green"
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
        "status": if record.state == ChangeState::Archived { "current" } else { "pending" },
        "action": "merge only after finalize — merging an unfinalized change orphans verification evidence",
        "head_tip_class": tip.tip_class,
    }));

    stages
}
pub(super) fn print_ship_status(
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
