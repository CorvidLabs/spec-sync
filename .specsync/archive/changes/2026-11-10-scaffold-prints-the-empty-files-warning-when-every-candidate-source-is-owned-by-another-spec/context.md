---
change: scaffold-prints-the-empty-files-warning-when-every-candidate-source-is-owned-by-another-spec
artifact: context
---

# Context

PR #1025 (Fixes #611), round 3. The corvid-agent review at `53c1fa5e` found that
REQ-cmd-scaffold-006 AC2, reworded in round 2, says an all-owned candidate set takes
the no-files path: `files: []` plus the empty-files warning. `specsync new foo`
prints that warning (`cmd_add_spec` in `src/commands/scaffold.rs`); `specsync
scaffold foo` wrote `files: []` and printed nothing on stderr.

The requirement already says what should happen, so no canonical spec changes: both
commands now call one shared warning helper, and the ownership integration tests
assert `files: []` and the warning on stderr.

Authorization: Leif's 2026-11-10 merge rule lets bug fixes that make behavior match
existing specs run the full lifecycle with 0xLeif as approve and review actor.
