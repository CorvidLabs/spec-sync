---
change: keep-check-fix-export-rows-in-the-table-that-already-holds-documented-exports-768-case-1
artifact: tasks
---

# Tasks

- [x] After approval, cherry-pick #815's src and test change (a305444a, without its `.specsync` and `specs` edits) with corvid-agent as author. Evidence: commit 40c6a4a0, author corvid-agent, cherry-picked from 0307d6f0, corvid-agent's rebased copy of a305444a; all three carry the same `src` and `tests` patch line for line, and a305444a is an ancestor of #815's head 80658a63.
- [x] Generalize #815's `assert_fix_768_table` to take the existing export row and expected width; keep `fix_768_uses_one_contiguous_export_table` passing. Evidence: commit af2e4e38; the helper takes the anchor row and width.
- [x] Add the layout (c) and (d) tests from `testing.md`, and confirm their labels against builds of origin/main 709ab2bd and #815 (80658a63). Evidence: commit af2e4e38, plus the approved header-rule and fallback tests; labels recorded in `testing.md`.
- [x] Replace #815's `export_table` selection in `src/commands/check.rs` with the REQ-cmd-check-017 order: last table holding a documented export row, else first export-like header that is not a parameter or field table, else the last table. Reuse the parser's row-symbol rule rather than a second definition. Evidence: commit 2e36b08b; a data row counts as documented when `parser::get_spec_symbols` reads a symbol from it (with any `####` headings above it), so `src/parser.rs` is unchanged.
- [x] Record #768 case 2 as an open task in `specs/cmd_check/tasks.md`, with the reason, and a lesson on a later plane of `specs/cmd_check/cmd_check.3md`. Evidence: commit 2e36b08b (open task, plane z=5).
- [x] Run the lanes in `testing.md` and record results. Evidence: `testing.md` Results.

## Human gates

Scope approval (`specsync change approve <id> --actor <human>`) by a human. Implementation review
by a human after a current `change check`. Finalization, merge and release are separate human
decisions. Closing #815 is the maintainer's call after approval.

Decisions proposed to the human, not made by this draft:

- **Last vs first documented table.** Proposed: last, so origin/main's placement is kept whenever
  the last table is an export table (for example a flat section with a Functions table then a
  Types table). Alternative: first.
- **Header rule list.** Proposed: `Name`, `Function`, `Export`, `Symbol`, singular or plural.
  `Type` is left out because this repository uses `| Type | Description |` for exported types and
  `| Name | Type | ... |` for parameters; such tables are still found by rule 1 once they hold a
  row.
- **Parameter or field table test.** Proposed: a non-first column named `Type`, or a first column
  named `Parameter`, `Param`, `Argument` or `Field`. The cost: an export table shaped
  `| Name | Type | Description |` with no documented rows yet falls through to the old behavior.
- **Fallback when nothing matches.** Proposed: origin/main's last table, now with width from that
  table. Alternative: the first table, which would be right more often but changes behavior.
- **Extra tests.** Proposed: one test where no table holds a documented row and the header rule
  picks the table, and one where the fallback applies.
- **Release.** Proposed: a later patch (6.1.3), not 6.1.2; see the PR body.
