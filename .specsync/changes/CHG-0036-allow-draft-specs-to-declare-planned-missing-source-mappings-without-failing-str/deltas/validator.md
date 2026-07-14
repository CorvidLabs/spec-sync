## ADDED

### REQUIREMENT REQ-validator-007

Validation SHALL treat safe missing draft file mappings as planned by default without adding nonexistent files to current coverage.

Acceptance Criteria

- Draft planned mappings pass strict validation with explicit notices.
- Activating the spec or enabling `require_draft_files` restores the missing-file error.
- Creating the file transitions it to normal mapping and coverage.
- Existing files retain containment, readability, and duplicate-ownership validation.
- Unsafe paths remain errors in every lifecycle status.
