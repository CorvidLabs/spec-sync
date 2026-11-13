---
change: the-watch-tests-read-a-run-through-to-the-line-after-its-verdict
artifact: context
---

# Context

Issue #1055, from t384's review of #1053. Item 1 (kill the child on every exit path) is already
met on main: #1053 merged `watch_until` with its child in `KillOnDrop`, whose `Drop` kills and
reaps it, and `Drop` also runs when a test panics or an assertion fails. Item 2 is this change:
`watch_until` stopped at the first verdict marker, so a later line of the same run could be
missed. `watch` prints `Watching for changes in: ...` after its first run (`src/watch.rs`) and
`Watching for changes...` after each later one, so the harness now waits for "Watching for
changes" after the verdict. The stop rule moves into `run_finished`, which a unit test checks on
sample output.
