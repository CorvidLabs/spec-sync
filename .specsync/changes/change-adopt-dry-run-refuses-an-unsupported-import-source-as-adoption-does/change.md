---
id: change-adopt-dry-run-refuses-an-unsupported-import-source-as-adoption-does
state: approved
type: bug_fix
base_commit: 18385ef5ead84f0b51d70a29a6fed8a3e5ca222a
---

# Change adopt --dry-run refuses an unsupported import source as adoption does

## Intent

change adopt --dry-run refuses an unsupported import source as adoption does

## Affected Canonical Specs

- `change`

## Acceptance Criteria

- change adopt --dry-run --source bogus exits nonzero with unsupported adoption source and prints no import plan, exactly as change adopt --source bogus does; a dry run with a supported or detected source still prints its plan and writes nothing

## No-spec Rationale

change adopt documents --dry-run as a preview of adoption and --source as openspec or speckit; the preview accepted any source and claimed it would import from it, while adoption refuses it as unsupported (#432)
