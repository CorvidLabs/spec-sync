# Lesson bundle: check-reports-a-module-s-3md-lesson-file-that-does-not-parse

Material for folding this change's lessons into each affected module's lesson file.
Synthesise from what actually happened below; do not restate the change description.

## What this change was

- **Title**: Check reports a module's .3md lesson file that does not parse
- **Kind**: Feature
- **Specs**: validator, cmd_check
- **Paths**: src/validator/consumed_by_re.rs, tests/integration/check.rs
- **Acceptance**: A spec whose <module>.3md does not parse gets one check warning naming the file, the parser's error and context.md, and check --strict fails; a .3md that parses, or none, adds no finding (red then green)

## Evidence

- Verification commit: `281624276a56f653f426afba36ebf7615bee9aa4`
- Base commit: `18385ef5ead84f0b51d70a29a6fed8a3e5ca222a`
- Verified by: `specsync check --spec cmd_check --spec validator`

## From the change's context.md

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

## From the change's design.md

# Design

The check sits with the other companion checks in `validate_spec_content_internal`
(`src/validator/consumed_by_re.rs`), under `validate_companions`, so it runs exactly where the
`requirements.md` duplicate-id check runs and nowhere a confined snapshot validates. It reads
`<spec dir>/<module>.3md` when that file exists, parses it with the same `threemd::parse` the
lesson reader uses, and on failure pushes one warning:
`<path> does not parse as 3md (<error>); its lessons are not read, and <spec dir>/context.md is used
instead`. A file that cannot be read gets the same warning with the read error. Warnings already
fail `--strict`, so nothing else changes.

## From the change's testing.md

# Testing

- Integration `an_unparseable_3md_is_a_check_warning_and_fails_strict` in
  `tests/integration/check.rs` (fails on main: no warning, and `--strict` passes).
- Integration `a_3md_over_the_read_bound_is_reported_and_not_read` in the same file (fails with a
  plain read: the file is read in full and nothing is reported), and
  `no_project_file_is_read_outside_the_bounded_read`, which names a plain read of the file.
- This repository's own `check --strict` still passes: every `.3md` here parses since #1062.

## Requirement evidence

| Requirement | Evidence |
|---|---|
| REQ-cmd-check-023 | `tests/integration/check.rs`: `an_unparseable_3md_is_a_check_warning_and_fails_strict`, `a_3md_over_the_read_bound_is_reported_and_not_read` |

## Where these lessons go

- `specs/validator/validator.3md`
- `specs/cmd_check/cmd_check.3md`
