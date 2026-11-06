# Lesson bundle: declare-the-three-paths-the-archived-status-latency-change-edited-but-did-not-list

Material for folding this change's lessons into each affected module's lesson file.
Synthesise from what actually happened below; do not restate the change description.

## What this change was

- **Title**: Declare the three paths the archived-status latency change edited but did not list
- **Kind**: BugFix
- **Specs**: change
- **Paths**: src/change.rs, src/change/sdd_version.rs, tests/integration/change.rs
- **Acceptance**: specsync change audit --strict passes on the #918 branch: src/change.rs, src/change/sdd_version.rs and tests/integration/change.rs are covered by a change (they were flagged as meaningful changed paths not covered)
- **Acceptance**: No code, test or spec text changes: the three files are byte-identical to the reviewed #918 archive tip 76ee339b
- **Acceptance**: CI Lifecycle gate and trust pass on the archived tip, so the PR can merge (traces SPEED-1 and SPEED-1.a, the intent the covered edits serve)

## Evidence

- Verification commit: `54aebec06b241a11afd55bc2e093fcd27d4e14b1`
- Base commit: `76ee339b8002eb8a86161bafc3360e99ff4015af`
- Verified by: `specsync check --spec change`

## From the change's context.md

# Context

#918 shipped `archived-change-status-no-longer-walks-history-per-archive-or-per-workflow-v1-change-and-read-commands-take-a-timeout` at archive tip `76ee339b`. CI's Lifecycle gate and trust run `specsync change audit --strict`, which failed: "meaningful changed paths are not covered by an active change: src/change.rs, src/change/sdd_version.rs, tests/integration/change.rs". The archived change declared ten paths but also edited these three:

- `src/change.rs`, +1: the `include!` for `change/read_deadline.rs`.
- `src/change/sdd_version.rs`, +11: `flush_change_read_scope_caches`, which flushes read-scope caches when a read times out.
- `tests/integration/change.rs`, +116: `an_archived_status_past_its_timeout_exits_124_and_names_how_to_resume`.

All three are in the PR diff Leif reviewed, and they implement REQ-change-137 and REQ-cmd-change-032. `change correct` can't amend affected paths, so Leif chose this covering change ("Same covering fix", relayed by orc t18 on 2026-11-06). It changes no code, tests or spec text.

## From the change's testing.md

# Testing

- `specsync change audit --strict` passes locally before the push; before this change it failed on the three paths.
- `git diff 76ee339b -- src/change.rs src/change/sdd_version.rs tests/integration/change.rs` is empty, so the reviewed bytes are unchanged.
- CI on the archived tip: Lifecycle gate, trust and the required gates pass.
- The integration test already in `tests/integration/change.rs`, `an_archived_status_past_its_timeout_exits_124_and_names_how_to_resume`, passed in the verify lane on `47ac3bb0` (2833 unit + 495 integration).

## Where these lessons go

- `specs/change/change.3md`
