---
id: generate-ownership-wording-matches-shipped-no-files-path
state: implementing
type: bug_fix
base_commit: b2432e0488846b92f869e307c78b672354073415
---

# Generate ownership wording matches shipped no-files path

## Intent

generate ownership wording matches shipped no-files path

## Affected Canonical Specs

- `generator`
- `cmd_scaffold`

## Acceptance Criteria

- REQ-generator-009 and REQ-cmd-scaffold-006 describe the shipped behavior: an all-owned candidate set takes the existing no-files path (spec written with files: [] plus the empty-files warning); new foo against an owned flat file is covered by a test; the requirements carry no contradiction about config.modules mappings

## No-spec Rationale

Not applicable
