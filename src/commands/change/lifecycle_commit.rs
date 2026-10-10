use super::repo_status::git_rev_parse;
use crate::change;
use crate::types::OutputFormat;
use std::path::Path;

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
pub(super) fn run_checked_commit(
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

    let mut disclosed = Vec::new();
    let materialized = git_commit_lifecycle(
        root,
        &change::lifecycle_commit_scope(root, &resolved)?,
        &format!("chore(lifecycle): materialize {resolved}"),
        true,
    )?;
    disclose_left_out(&materialized.left_out, &mut disclosed, Some(&resolved));
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
    say("Committed the verification evidence.");

    if push {
        run_git(root, &["push"])?;
        say("Pushed.");
    }
    say("Verified, committed, and consistent with the committed tree.");
    Ok(())
}

pub(super) fn run_git(root: &std::path::Path, args: &[&str]) -> Result<(), String> {
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

/// What one lifecycle commit did.
pub(super) struct LifecycleCommit {
    /// False when nothing was staged, so no commit was made.
    pub(super) committed: bool,
    /// Untracked files deliberately left out, as project-relative paths.
    pub(super) left_out: Vec<String>,
}

/// Stage what a lifecycle commit owns and commit it, treating "nothing to commit" as success so
/// the sequence stays idempotent when a pass produced no change.
///
/// `scope` is [`change::lifecycle_commit_scope`] for the change being committed. This used to be
/// `git add -A`, which put every untracked, non-ignored file in the project into a commit that
/// `--push` then published. See [`stage_lifecycle_paths`] for what is staged now.
pub(super) fn git_commit_lifecycle(
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
    let status = std::process::Command::new("git")
        .args(["diff", "--cached", "--quiet"])
        .current_dir(root)
        .status()
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
pub(super) fn stage_lifecycle_paths(
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
    let output = std::process::Command::new("git")
        .args([
            "status",
            "--porcelain=v1",
            "-z",
            "--untracked-files=all",
            "--",
            ".",
        ])
        .current_dir(root)
        .output()
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
pub(super) fn path_is_within(path: &str, scope: &str) -> bool {
    let scope = scope.trim_end_matches('/');
    path == scope
        || path
            .strip_prefix(scope)
            .is_some_and(|rest| rest.starts_with('/'))
}

pub(super) fn git_output_text(root: &Path, args: &[&str]) -> Result<String, String> {
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
    Ok(String::from_utf8_lossy(&output.stdout).into_owned())
}

/// How many left-out paths a warning names before it summarizes the rest.
pub(super) const LEFT_OUT_LISTING_LIMIT: usize = 20;

/// Name, on stderr, the untracked files a lifecycle commit left out. `disclosed` carries what an
/// earlier commit of the same run already named, so the second commit does not repeat it.
///
/// Stderr, like the ledger note: it must survive `--quiet` and keep `--format json` stdout a
/// single document. `verified_change` is set when the files sit under recorded verification, which
/// digests the working tree including them.
pub(super) fn disclose_left_out(
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

pub(super) fn left_out_warning(paths: &[&String], verified_change: Option<&str>) -> String {
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
