use super::repo_status::git_rev_parse;
use crate::change;
use crate::change::ChangeRecord;
use std::fs;
use std::path::Path;

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
pub(super) struct ArchiveCandidate {
    pub(super) id: String,
    pub(super) state: String,
    /// Verifying, with review and verification files on disk. The ship command is the next step.
    pub(super) ship: bool,
}

pub(super) fn ordered_archive_id(open: &[ArchiveCandidate]) -> Option<String> {
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
pub(super) fn archive_sentence_without_command(sentence: &str) -> String {
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
pub(super) fn pull_request_archive_sentence(open: &[ArchiveCandidate]) -> String {
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

pub(super) fn open_archive_candidates(root: &Path) -> Result<Vec<ArchiveCandidate>, String> {
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
/// A blocking review, a legacy workflow, or a review recorded against a different
/// commit cannot be archived by that command. The same proof is in
/// `.github/scripts/archive-readiness.sh`: verdict `pass`, workflow v2, and
/// `implementation_commit` equal to HEAD. Anything the check cannot prove stays
/// on `change status`.
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
    git_rev_parse(root, "HEAD")
        .ok()
        .is_some_and(|head| head == commit)
}

pub(super) fn pull_request_sentence(root: &Path) -> Result<String, String> {
    Ok(pull_request_archive_sentence(&open_archive_candidates(
        root,
    )?))
}

pub(super) fn ship_next_action(
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
pub(super) fn lessons_next_action(root: &Path, id: &str, archive: &Path) -> String {
    let _ = (change::lesson_fold_targets(root, id), archive);
    "Push this archive, wait until the checks are green, then merge the pull request.".to_string()
}
