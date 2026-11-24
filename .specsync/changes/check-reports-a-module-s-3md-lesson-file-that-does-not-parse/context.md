---
change: check-reports-a-module-s-3md-lesson-file-that-does-not-parse
artifact: context
---

# Context

Issue #1060. When `specs/<module>/<module>.3md` does not parse, SpecSync ignored it silently:
`module_lesson_lines` falls back to `context.md`, every 3md lesson for the module disappears from
briefings, and `check --strict` still passed. It happened on main: `cmd_scaffold.3md` had two
planes at `z=5` (fixed by #1062, "part of #1060"). This change is the reporting half.

Leif selected "#1060 broken .3md reported" on 2026-11-13T03:28Z, and the coordinator
(claude-code-t370) assigned REQ-cmd-check-018. Main took 018 to 022 for the hash-cache and
project-read requirements before this change shipped, so on 2026-11-24 the ID was renewed to
REQ-cmd-check-023 and the Error Cases delta was carried onto main's two new rows; the requirement's
words did not change. The briefing still falls back silently, as
REQ-change-106 allows; `check` is where an author sees and fixes the file. Lifecycle commands never
fail on it.
