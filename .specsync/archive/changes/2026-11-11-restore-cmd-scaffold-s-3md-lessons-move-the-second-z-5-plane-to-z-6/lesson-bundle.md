# Lesson bundle: restore-cmd-scaffold-s-3md-lessons-move-the-second-z-5-plane-to-z-6

Material for folding this change's lessons into each affected module's lesson file.
Synthesise from what actually happened below; do not restate the change description.

## What this change was

- **Title**: Restore cmd_scaffold's 3md lessons: move the second z=5 plane to z=6
- **Kind**: BugFix
- **Specs**: cmd_scaffold
- **Paths**: specs/cmd_scaffold/cmd_scaffold.3md
- **Acceptance**: specs/cmd_scaffold/cmd_scaffold.3md has unique plane positions (z=0 to z=6) and parses, so a change briefing for cmd_scaffold reads its lessons from cmd_scaffold.3md instead of falling back to context.md; check --strict still passes

## Evidence

- Verification commit: `a642b549d580a780381e2987ece7eb77f88359c2`
- Base commit: `ad8544e9da79725f499b38ae4d4b3a052be5f807`
- Verified by: `specsync check --spec cmd_scaffold`

## From the change's context.md

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

## From the change's testing.md

# Testing

Bug-fix, no spec change. Observable before and after, using the branch build of main
`ad8544e9`:

- Red, at main with the duplicate: in a throwaway worktree, `specsync change new … --spec
  cmd_scaffold` briefs "Lessons: … specs/cmd_scaffold/context.md (13 line(s))". The 3md file
  did not parse, so the fallback was used.
- Green, with the second plane at `z=6`: the same command briefs "Lessons: …
  specs/cmd_scaffold/cmd_scaffold.3md (16 line(s))".
- Planes are now z=0 to z=6, unique and in order. `specsync check --strict --require-coverage
  100` passes, and `change check --commit` runs last. The full suite runs in hosted CI.

No new automated test: the general guard, which reports an unparseable 3md, is #1060's scope.

## Where these lessons go

- `specs/cmd_scaffold/cmd_scaffold.3md`
