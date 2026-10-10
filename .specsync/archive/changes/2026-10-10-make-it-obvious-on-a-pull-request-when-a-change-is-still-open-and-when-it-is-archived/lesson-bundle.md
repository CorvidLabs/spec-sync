# Lesson bundle — make-it-obvious-on-a-pull-request-when-a-change-is-still-open-and-when-it-is-archived

Material for folding this change's lessons into the affected specs' `context.md`.
Synthesise from what actually happened below; do not restate the change description.

## What this change was

- **Title**: Make it obvious on a pull request when a change is still open and when it is archived
- **Kind**: Feature
- **Specs**: cmd_change
- **Paths**: src/commands/change.rs, src/commands/change.rs, .github/workflows/ci.yml, .github/scripts/archive-readiness.sh, .github/scripts/test-archive-readiness.sh, specs/cmd_change/cmd_change.spec.md, specs/cmd_change/requirements.md, specs/cmd_change/context.md, specs/cmd_change/tasks.md, specs/cmd_change/testing.md, hi/ship.md, INTENT.md
- **Acceptance**: When I look at a pull request, I can tell whether its change is still open or already archived.
- **Acceptance**: A pull request that still has an open change does not look finished.
- **Acceptance**: A pull request whose changes are all archived says it is ready to merge once the checks are green.
- **Acceptance**: When it is time to archive, one command archives the change, commits that archive, and tells me that merging is the step that is left.
- **Acceptance**: When it is not time, that same place tells me the one reason and the one command.
- **Acceptance**: When more than one change is open, I am told which one to archive first.

## Evidence

- Verification commit: `7fb084f7c39b80e91743753a38e3b289f5a5c6f9`
- Base commit: `ee89640c3ccd3c68d044ff0a58499749b2fba92c`
- Verified by: `specsync check --spec cmd_change`

## From the change's context.md

# Context

A pull request with a verified open change and a pull request whose change is archived both pass the lifecycle gate. The difference shows up only if you look in `.specsync/changes/` or `.specsync/archive/`.

`change ship` finalizes and then tells you to commit, push, wait, and merge. The archive commit is a second step. The next-action line mentions merging even when a change is still open.

The Archive check is the sentence on the pull request. When the change is ready, `change ship` archives it and commits that archive. When it is not ready, status and ship name one reason and one command.

## From the change's design.md

# Design

The Archive job is always selected. Its script reads `.specsync/changes/*/state.json` and does not build Rust. An unreadable state file still counts as open.

The change closest to archive goes first: verifying, then approved, then implementing, then draft, then anything else, and within a state the lowest id. A verifying change that already has review and verification evidence is told to run ship for that id. Every other open change is told to run status for that id.

`change ship` commits with the archive lifecycle message after finalize. Passing `--push` still pushes that commit. It does not commit twice.

## From the change's testing.md

# Testing

REQ-cmd-change-019

- `.github/scripts/test-archive-readiness.sh` covers an open change, several open changes, and no open changes.
- `ship_commits_the_archive_and_names_merge_as_what_is_left` in `src/commands/change.rs`.
- `status_says_which_open_change_to_archive_first` in `src/commands/change.rs`.
- `python3 .github/scripts/test-required-ci-gate.py` still passes with the Archive job on the required gate.

## Where these lessons go

- `specs/cmd_change/context.md`
