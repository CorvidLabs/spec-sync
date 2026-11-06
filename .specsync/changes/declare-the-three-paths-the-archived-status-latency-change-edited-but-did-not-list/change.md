---
id: declare-the-three-paths-the-archived-status-latency-change-edited-but-did-not-list
state: implementing
type: bug_fix
base_commit: 76ee339b8002eb8a86161bafc3360e99ff4015af
---

# Declare the three paths the archived-status latency change edited but did not list

## Intent

Declare the three paths the archived-status latency change edited but did not list

## Affected Canonical Specs

- `change`

## Acceptance Criteria

- specsync change audit --strict passes on the #918 branch: src/change.rs, src/change/sdd_version.rs and tests/integration/change.rs are covered by a change (they were flagged as meaningful changed paths not covered)
- No code, test or spec text changes: the three files are byte-identical to the reviewed #918 archive tip 76ee339b
- CI Lifecycle gate and trust pass on the archived tip, so the PR can merge (traces SPEED-1 and SPEED-1.a, the intent the covered edits serve)

## No-spec Rationale

No spec text changes: the three paths already carry the reviewed #918 edits (the read_deadline include, the read-scope cache flush on timeout, and the timeout integration test) under REQ-change-137 and REQ-cmd-change-032; this change only declares them so the lifecycle audit covers them
