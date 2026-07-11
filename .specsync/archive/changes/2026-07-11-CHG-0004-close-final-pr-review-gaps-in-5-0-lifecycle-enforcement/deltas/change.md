## ADDED

### REQUIREMENT REQ-change-012
The lifecycle SHALL fail closed across coverage, persisted closing evidence, semantic-delta validation, dependency ordering, and supported canonical version formats.

Acceptance Criteria
- Only implementing, verifying, or accepted changes cover meaningful delivery paths.
- Local coverage includes committed, staged, unstaged, and untracked meaningful paths.
- Accepted workspaces require fresh successful verification and matching closing approval evidence.
- Delta modules, operation headings, tombstones at acceptance, and transitive dependency order are validated deterministically.
- Integer and semantic spec versions advance without losing their format.
