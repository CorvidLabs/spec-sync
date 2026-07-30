---
change: CHG-0068-stabilize-specsync-6-0-with-a-low-churn-normal-workflow-preserved-audited-guara
artifact: testing
---

# Testing

## Characterizations

- Reproduce PR #455's false gate: implementation checks pass while strict coverage rejects the
  stale branch because no active change owns its paths.
- Model Rune PR #23 as eight changes that each finalized on its originating PR and assert no
  follow-up cleanup PR or duplicate product matrix is required.

## Focused tests

- One workflow: one approval, no lifecycle-mode grammar, no closing gate, one active/archive layout, and
  byte-compatible reads of existing two-approval history.
- Approval boundary: execution/test/evidence/delta/materialization/lifecycle changes preserve scope
  approval but stale execution evidence; added intent, criteria, affected scope, dependencies, or
  supersession obligations require renewal with an exact plain-language diff.
- Verification persistence: a null commit remains current only for a genuinely non-Git project;
  introducing Git without commit-bound evidence fails closed.
- Status: every state emits exactly one next action; newcomer fixtures traverse
  new→approve→implement→check→review→finalize→GitHub merge.
- Finalizer: stale approval, incomplete tasks, failed targeted checks, changed implementation tree,
  wrong PR/head, digest replay, unsafe archive paths, and non-idempotent retries fail before writes.
- Archive-only classification: the parent implementation commit has required green checks; the
  child diff contains only exact approved lifecycle/archive paths; code/spec/requirements/tests and
  their bidirectional relationships are identical; archive integrity passes; required CI receives
  success; full product jobs are not selected.
- Scoped review: agent-authored implementation needs one reviewer attestation bound to the parent
  package/diff/delta/evidence digest; implementation changes stale it; archive-only changes do not
  rerun or stale it; status explains how to request the check.
- Strict policy: explicit `--strict`, project policy, and release/security classification fixtures
  add full-history/full-suite/security/release validators to the same evidence and never alter the
  state machine, approval count, layout, commands, finalization, archive, or status path.
- Snapshot bounds: deterministic Git command counts, stable graph order, one owner-batch scan, and
  warm/cold semantic parity.
- Selected fixes: schema DDL replay/errors, visible ignore suppression, TypeScript/Erlang fixtures,
  worktree/core.hooksPath/project-keyed managed blocks, and customized agent artifact conflicts.

## Requirement evidence

- `REQ-change-043`, `REQ-cli-008`, `REQ-cli-args-009`, `REQ-cmd-change-005`: one-workflow grammar,
  task-progress digest stability, strict-validator status, and next-action tests.
- `REQ-change-044`, `REQ-github-005`: exact finalization classifier, parent-check reuse,
  finalization-digest, archive integrity, post-merge binding, and release-wait tests.
- `REQ-change-045`: bounded snapshot query-count, deterministic graph, warm/cold parity, and
  one-pass owner-batch tests.
- `REQ-change-046`: missing/current/stale independent-review and review-only-child reuse tests.
- `REQ-agents-004`, `REQ-hooks-002`: manifest/digest conflict aggregation, exact uninstall,
  worktree/submodule/core.hooksPath, symlink, idempotence, and project-keyed block tests.
- `REQ-schema-001`, `REQ-validator-009`, `REQ-commands-005`, `REQ-cmd-check-003`: ordered checked
  snapshot, DDL precondition/collision, canonical identity, fail-closed/vacuous schema, and
  structured suppression tests.
- `REQ-ignore-001`: category/path parsing, visible diagnostics, structured suppression, and strict
  unsuppressed-count tests.
- `REQ-exports-006`: TypeScript Unicode declaration/re-export and Erlang regex/AST arity parity
  fixtures.

## Final gate

Run `fledge lanes run verify`, the full repository lane, strict 100% spec coverage, score ≥80,
`fledge trust verify`, release validators, and the private sandbox workflow once after focused
iteration is green. Separately prove that a pure finalization child selects only the lightweight
archive lane.
