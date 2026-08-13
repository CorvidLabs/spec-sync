---
id: CHG-0112-a-tree-with-source-and-no-specs-must-show-its-coverage-number-and-must-not-pass
state: implementing
type: bug_fix
base_commit: 22a6a49027b4d2569501f8b494e886a8a1628a49
---

# A tree with source and no specs must show its coverage number and must not pass strict validation

## Intent

A tree with source and no specs must show its coverage number and must not pass strict validation

## Affected Canonical Specs

- `cmd_check`

## Acceptance Criteria

- Running `specsync check` on a project with source files and an empty specs directory prints the coverage figures rather than omitting them, and `--strict` exits non-zero instead of reporting the tree clean. The machine-readable payload carries the source-file count and coverage percent so a consumer can distinguish a project with unmeasured source from an empty one. A project with no source files at all, whose specs have simply not been generated yet, continues to exit zero under `--strict`.

## No-spec Rationale

Not applicable
