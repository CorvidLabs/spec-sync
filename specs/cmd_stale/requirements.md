---
spec: cmd_stale.spec.md
---

## User Stories

- As a developer, I want to know which specs have drifted from their source files so I can update them
- As a CI operator, I want staleness checks integrated into the validation pipeline so drift is caught early

## Acceptance Criteria

- `specsync stale` lists specs whose source files have changed since the spec was last modified
- Reports include commit count, changed file list, and last commit details
- JSON output mode (`--format json`) produces machine-readable staleness data
- `specsync check --stale` integrates drift warnings into the standard check pipeline
- Exit code is non-zero when stale specs are detected (for CI usage)

## Constraints

- Must not panic on expected error conditions — return Results or print and exit
- Must work with the project's Clap-based CLI argument parsing
- Git operations must handle missing git repos gracefully (non-git directories)
