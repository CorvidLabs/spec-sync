---
id: CHG-0028-use-the-release-candidate-specsync-binary-in-the-trust-workflow
state: archived
type: operations
base_commit: c98d29810f78abcdd6a2fec9b137667d3ab2fc5b
---

# Use the release candidate SpecSync binary in the Trust workflow

## Intent

Use the release candidate SpecSync binary in the Trust workflow

## Affected Canonical Specs

- None

## Acceptance Criteria

- The Trust workflow builds the pull request binary and packages it in a checksum-verified runner-local mirror while hosted Trust passes without weakening lifecycle contract risk or provenance enforcement.

## No-spec Rationale

This changes only release-validation workflow wiring; it does not change the SpecSync product contract.
