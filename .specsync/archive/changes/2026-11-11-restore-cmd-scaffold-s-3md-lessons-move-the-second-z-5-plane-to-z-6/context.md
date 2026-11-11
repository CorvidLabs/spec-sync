---
change: restore-cmd-scaffold-s-3md-lessons-move-the-second-z-5-plane-to-z-6
artifact: context
---

# Context

#1025 (`f8d66ad1`, merged 2026-11-10) added the "Scaffold never steals an owned file (#611)"
lesson to `specs/cmd_scaffold/cmd_scaffold.3md` at `z=5`, a position the "Exact source
selection" plane already held. threemd 2.2.1's `parse` refuses two planes at one position
(`ParseError::DuplicatePlane`). Lesson reading (`module_lesson_lines`) then falls back to
`context.md` without a word, so every cmd_scaffold 3md lesson stopped reaching briefings, and
`check --strict` still passed. It was found in the 2026-11-11 main health report, as the only
duplicate position across all 62 3md files.

This change renumbers the second plane to `z=6` and touches nothing else. Reporting an
unparseable 3md file is #1060 itself and stays out of this change. Lesson planes are
append-only: keep every plane's text and order.
