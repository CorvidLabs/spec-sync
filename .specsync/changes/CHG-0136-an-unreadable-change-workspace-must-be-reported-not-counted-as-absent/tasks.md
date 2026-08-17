---
change: CHG-0136-an-unreadable-change-workspace-must-be-reported-not-counted-as-absent
artifact: tasks
---

# Tasks

- [x] Reproduce from scratch: two healthy changes, corrupt one, confirm `list`/`status` report
      the project empty at exit 0 while `show` on the healthy id still works.
- [x] Reproduce #603 end to end: 6.0 `change new`, 5.2 write, confirm the record loses both
      version fields, then confirm what 6.0 does *afterwards* — the step the issue never took.
- [x] Establish that 6.0 already detects the downgrade, and that `list`/`status` discard it, so
      the fix is not a version-stamp change.
- [x] Enumerate every site that reads the roster before touching any of them. Four production
      swallow sites, not one.
- [x] Introduce `ChangeRoster` / `UnreadableChange` so the two facts cannot share a value.
- [x] Stop enumeration aborting on the first bad workspace; collect per-workspace failures.
- [x] Keep directory-level failures as hard errors — no partial truth exists there.
- [x] Retain `list_changes_checked` as a fail-closed adapter so the eleven internal digest and
      ledger callers keep their historical contract.
- [x] Fix all four swallow sites, including the three not named in the bug report.
- [x] Make `ship` and lifecycle commit resolution refuse to infer a target from a partial roster.
- [x] Make `sibling_active_change_ids` count unreadable workspaces as active.
- [x] Keep JSON a single parseable document in both shapes; avoid the double-document trap in
      `cmd_change`'s tail error handler.
- [x] Add a regression test with an explicit vacuity control.
- [x] Verify against gate 055 on both an unfixed and a fixed binary.
- [x] Whole-board check: exactly one drill may change state.
- [x] `cargo test`, `cargo clippy -- -D warnings`, `cargo fmt --check`.
- [x] CHANGELOG entry.
- [x] Semantic deltas for both affected modules; do not hand-edit `specs/`.
