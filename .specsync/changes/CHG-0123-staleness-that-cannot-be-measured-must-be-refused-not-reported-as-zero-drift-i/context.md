---
change: CHG-0123-staleness-that-cannot-be-measured-must-be-refused-not-reported-as-zero-drift-i
artifact: context
---

# Context

On a tree whose specs are six commits behind their source, with `.git` removed
and the file tree otherwise byte-identical:

    stale   -> "Not a git repository — staleness detection requires git history."  exit 1
    report  -> "Modules: 1 total, 0 stale"  JSON "stale": false, "commits_behind": 0  exit 0

Two commands, one tree, opposite answers. `stale` refuses a question it cannot
answer; `report` answers it with a zero it never measured. An unborn HEAD
(`git init`, no commits) produces the same split.

The cause is that there was no shared staleness reader. Four implementations
computed the same spec-behind-source distance from `git_last_commit_hash` and
`git_commits_since`, and only one guarded the precondition:

    stale.rs:26      correct — `!is_git_repo(root) || !has_commits(root)`, exits 1
    report.rs:105    no guard at all
    check.rs:512     `is_git_repo` only, never `has_commits`
    scoring.rs:606   `is_git_repo` only, never `has_commits`

`stale.rs`'s guard carries a comment citing #558. That fix landed there and
nowhere else — the third time in this campaign a correction reached the site in
the bug report while parallel implementations kept the defect, after #562
(fixed in `output.rs` only) and #570 (fixed at `load_and_discover` only).

The scoring site is the one with the sharpest consequence. Measured on two trees
differing only in whether `.git` exists:

    no git   Fresh 20/20  -> 80/100 [B]
    6 behind Fresh 17/20  -> 77/100 [C]

Deleting the repository raises the grade a full letter, and `--min-score` gates
on that number. That is filed separately as #586 and is closed by this change.

Ruled out: guarding only `report`, which was the command in the report. That
leaves three readers wrong and a fourth free to appear.
