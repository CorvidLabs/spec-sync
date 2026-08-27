---
change: audit-every-module-context-md-against-the-source-and-correct-every-false-or-stale-claim
artifact: tasks
---

# Tasks

- [x] Enumerate all 62 `specs/*/context.md` and extract every backticked token, classified as
      path / symbol / requirement ID, checked against the tree.
- [x] Read all 62 files in full and extract every checkable claim.
- [x] Verify each claim by running a command or reading the source; record the command.
- [x] Recount every stated number independently, with an explicit denominator.
- [x] Re-verify each finding before editing from it.
- [x] Fix dead pointers: `check_project_quiet`, `auto_regen_stale_specs`, `remove_section`,
      `src/exports.rs` (4 files, each also naming the wrong function), `build_schema` in
      `validator.rs`, `load_config` in `commands`, `is_suppressed` call sites in `ignore`.
- [x] Fix decayed counts: tracked `.md` files, archived approval ledgers, unit/integration test
      totals (3 files), `src/exports/` file count, `report` / `diff` / `coverage` integration-test
      counts, `cmd_init` inline test count.
- [x] Fix reversed behaviour: the change-sequence ledger is written by
      `floor_sequence_ledger_to_committed`; `deps` records rather than swallows unreadable input;
      `registry` parses with the `toml` crate; `output` does file I/O in `print_diff_markdown`;
      `deps` diagram mode validates and gates; zero-denominator coverage is `null`, not 100%.
- [x] Mark stale present tense as history: `parser.rs` and CRLF, CHG-0063 and CHG-0066 under
      verification, the CI lifecycle reimplementation deleted by #499.
- [x] Correct incomplete surface lists that read as complete: `git_utils` public surface and
      `StaleInfo`, `merge` public API and `results_to_json` statuses, `cmd_coverage` JSON keys,
      `DepsReport`, `ignore` aliases, `agents` command set, `comment` dependencies.
- [x] Leave judgement, rationale and attributed historical measurement untouched; record in
      `research.md` what was examined and deliberately not changed.
- [x] Record in `docs.md` the five defects found outside this change's scope, each with the
      evidence that established it, rather than widening scope to fix them.
- [x] State plainly in `testing.md` that no test is possible for a prose-only change, instead of
      adding one that cannot fail for the right reason.
- [x] Freeze scope at `change new` with the exact 34 specs and 34 paths.
- [x] `cargo fmt --check`, `cargo clippy -- -D warnings`, `cargo test` (2407 unit + 407
      integration) green.
