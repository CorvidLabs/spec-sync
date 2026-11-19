---
change: check-reports-a-module-s-3md-lesson-file-that-does-not-parse
artifact: requirements
---

# Requirements

REQ-cmd-check-018 is added (`deltas/cmd_check.md`): `check` warns on a module's
`<module>.3md` that cannot be read or parsed, naming the file, the parser's error and that
`context.md` is used instead, so `--strict` fails. A parsing `.3md`, or none, adds no finding.
REQ-change-106 is unchanged: lifecycle reads still fall back and never fail.

The validator spec's Error Cases table gains the matching row (`deltas/validator.md`), because
the check lives in the validator.
