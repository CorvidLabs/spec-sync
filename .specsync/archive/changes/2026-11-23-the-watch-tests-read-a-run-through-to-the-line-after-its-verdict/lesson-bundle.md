# Lesson bundle: the-watch-tests-read-a-run-through-to-the-line-after-its-verdict

Material for folding this change's lessons into each affected module's lesson file.
Synthesise from what actually happened below; do not restate the change description.

## What this change was

- **Title**: The watch tests read a run through to the line after its verdict
- **Kind**: BugFix
- **Paths**: tests/integration/watch.rs, CHANGELOG.md
- **Acceptance**: watch_until stops only once a verdict marker is followed by the line watch prints after each run (Watching for changes in: after the first, Watching for changes... after later ones); a unit test on sample output fails on the verdict alone and passes with the line after it (red then green)

## Evidence

- Verification commit: `2b703b4ad37aa6646c0eb7d80d3fc06f25d1c5c0`
- Base commit: `18385ef5ead84f0b51d70a29a6fed8a3e5ca222a`
- Verified by: `specsync check (no spec in scope)`

## From the change's context.md

# Context

Issue #1055, from t384's review of #1053. Item 1 (kill the child on every exit path) is already
met on main: #1053 merged `watch_until` with its child in `KillOnDrop`, whose `Drop` kills and
reaps it, and `Drop` also runs when a test panics or an assertion fails. Item 2 is this change:
`watch_until` stopped at the first verdict marker, so a later line of the same run could be
missed. `watch` prints `Watching for changes in: ...` after its first run (`src/watch.rs`) and
`Watching for changes...` after each later one, so the harness now waits for "Watching for
changes" after the verdict. The stop rule moves into `run_finished`, which a unit test checks on
sample output.

## From the change's testing.md

# Testing

- Unit `a_watch_run_is_finished_only_at_the_line_after_its_verdict` in
  `tests/integration/watch.rs` (fails on main: the verdict alone counts as finished).
- `watch_does_not_claim_a_pass_over_zero_specs` and the other watch tests still pass.

## Where these lessons go

This change declared no affected specs, so there is no module context to fold into.
