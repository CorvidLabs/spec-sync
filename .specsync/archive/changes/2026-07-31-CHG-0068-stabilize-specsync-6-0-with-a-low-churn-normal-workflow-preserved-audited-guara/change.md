---
id: CHG-0068-stabilize-specsync-6-0-with-a-low-churn-normal-workflow-preserved-audited-guara
state: archived
type: feature
base_commit: fc091c88f72a6d2fb2df168f4baa4370579ff8a2
---

# Stabilize SpecSync 6.0 with one scope approval, same-PR finalization, lightweight archive CI, scoped review, and selected UX fixes

## Intent

Stabilize SpecSync 6.0 with one scope approval, same-PR finalization, lightweight archive CI, scoped review, bounded lifecycle validation, and selected high-confidence UX fixes

## Affected Canonical Specs

- `change`
- `cmd_change`
- `cli_args`
- `github`
- `schema`
- `ignore`
- `commands`
- `exports`
- `hooks`
- `agents`
- `cli`
- `cmd_check`
- `validator`
- `cmd_comment`
- `cmd_coverage`
- `cmd_generate`

## Acceptance Criteria

- Every change follows one file layout and one workflow: create a change package, receive one human scope approval, implement code/specs/tests, pass bidirectional checks and ordinary PR review, finalize into the same PR archive, then merge through GitHub protections.
- No user-facing lifecycle mode, second SpecSync approval, alternate archive layout, or SpecSync merge command exists; explicit `--strict`, project policy, and release or security classification add powerful full-history, full-suite, security, or release validators to the same evidence only.
- Every `specsync change status` result prints exactly one explicit next action and teaches the core path `change new` → `change approve` → implement → `change check` → ordinary PR review → `change finalize` → GitHub merge.
- Same-PR finalization produces a metadata/archive-only commit in the existing dated archive layout and never requires a follow-up archive PR.
- The archive-only CI lane does not rerun the product test matrix; it proves the parent implementation commit has required green checks, permits only approved lifecycle/archive path changes, confirms code/spec/tree relationships are unchanged, validates archive integrity and bidirectional ownership, and reports to the required CI gate.
- Agent-authored changes receive one independent scoped review of the change package, implementation diff, canonical spec delta, and targeted evidence before finalization; the review is not repeated continuously or for the archive-only commit, and status explains how to request it.
- Lifecycle validation reuses one invocation snapshot, bounds Git/evidence queries, orders graphs deterministically, and validates owner batches in one pass without weakening bidirectional or historical integrity.
- Schema replay, ignore suppression, TypeScript and Erlang export discovery, and hook or agent installation fixes are selectively carried forward with focused regression coverage.
- Rune PR 23 is the regression proving eight accepted changes finalize in their originating PRs without a 103-file follow-up cleanup PR or a duplicate full CI cycle.
- One final release validation passes repository lanes, strict 100 percent spec coverage, score at least 80, trust verification, and the private CorvidLabs spec-sync-sandbox workflow before the 6.0 PR is ready.

## No-spec Rationale

Not applicable
