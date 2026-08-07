## MODIFIED

### REQUIREMENT REQ-change-012

The lifecycle SHALL fail closed across coverage, canonical persisted closing evidence, semantic-delta validation, dependency ordering, and supported canonical version formats.

Acceptance Criteria
- Only implementing, verifying, or terminal changes cover their own meaningful delivery paths; archived packages present in the current delivery (same-PR finalize tips) cover their affected_paths for path coverage even when no active change remains; only closing-valid accepted or authenticated archived changes can satisfy successor evidence.
- Local coverage includes committed, staged, unstaged, and untracked meaningful paths.
- Active accepted workspaces require successful verification, matching closing approval, and recursive exact-or-successor-covered current-input validity; archives require authenticated historical integrity and enter current-input recursion only when selected as successors.
- Delta modules, operation headings, tombstones at acceptance, and transitive dependency order are validated deterministically.
- Integer and semantic spec versions advance without losing their format.
