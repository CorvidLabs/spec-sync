---
change: never-read-a-project-file-through-a-link-that-leaves-the-project-a-non-regular-file-or-past-a-size-bound
artifact: requirements
---

# Requirements

Human intent: SETUP-7, its wording pending from the human (proposed: "A command I run in a repository somebody else wrote never reads outside it through a link it found there, and no file there can make it hang or run out of memory").

Added, one per affected module, in `deltas/`:
- `REQ-util-003` holds the bounded project read and the bounds.
- `REQ-change-161` covers `sdd.json`, the change records and the journal, which is the tester's finding.
- `REQ-hash-cache-008` covers `hashes.json` and hashing.
- `REQ-config-016`, `REQ-ignore-002`, `REQ-registry-004`, `REQ-mcp-012`, `REQ-agents-012` and `REQ-hooks-007` cover the state and hook files.
- `REQ-exports-011`, `REQ-deps-004`, `REQ-schema-002` and `REQ-validator-049` cover the source, schema and requirements reads.
- The spec readers are `REQ-archive-002`, `REQ-cmd-change-038`, `REQ-cmd-check-019`, `REQ-cmd-diff-005`, `REQ-cmd-init-009`, `REQ-cmd-lifecycle-005`, `REQ-cmd-merge-005`, `REQ-cmd-migrate-006`, `REQ-commands-014`, `REQ-cmd-report-006`, `REQ-cmd-resolve-003`, `REQ-cmd-stale-005`, `REQ-compact-002`, `REQ-generator-011`, `REQ-merge-007`, `REQ-output-007`, `REQ-scoring-007` and `REQ-view-002`.
