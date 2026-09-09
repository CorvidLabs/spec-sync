---
spec: cmd_generate.spec.md
---

## User Stories

- As a developer, I want deterministic local scaffolds for uncovered modules.
- As a developer, I want batch generation for selected modules.
- As an agent integrator, I want stable JSON paths for later refinement.

## Constraints

- Existing specs are never overwritten.
- Agent integrations may refine generated markdown outside this command.

### REQ-cmd-generate-001

The generate command SHALL create deterministic local specs only from trustworthy discovery.

Acceptance Criteria

- All generation modes use checked coverage discovery before selecting output.
- Malformed Gradle/manifest discovery exits nonzero before mutation.
- JSON mode remains parseable with `valid: false`, `inconclusive: true`, an explicit error, and an
  empty `generated` collection.

### REQ-cmd-generate-002

`specsync generate` SHALL fail closed when it skips every unspecced module because no source files were found, instead of exiting 0 with an empty write set.

Acceptance Criteria
- Text mode lists the skipped module names and exits 1.
- JSON mode includes `skipped_no_files` and exits 1 when `generated` is empty and that list is not.
- Batch generate (`--batch`) uses the same rule.
- A fully covered project still prints `No specs to generate` and exits 0.

