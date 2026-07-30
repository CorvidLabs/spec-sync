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
  approval but stale execution evidence; added, removed, or replaced intent, criteria, affected
  scope, dependencies, or supersession obligations require renewal with an exact plain-language
  diff. The one legacy CHG-0068 adoption is frozen by exact code/commit/blob/scope/classification
  allowlists and explicitly claims no unavailable preimage equivalence.
- Verification persistence: a null commit remains current only for a genuinely non-Git project;
  introducing Git without commit-bound evidence fails closed.
- Status: every state emits exactly one next action; newcomer fixtures traverse
  new→approve→implement→check→review→finalize→GitHub merge.
- Finalizer: stale approval, incomplete tasks, failed targeted checks, changed implementation tree,
  wrong PR/head, digest replay, unsafe archive paths, and non-idempotent retries fail before writes;
  a crash after directory rename resumes from the accepted dated destination.
- Archive-only classification: the parent implementation commit has required green checks; the
  child diff contains only exact approved lifecycle/archive paths; code/spec/requirements/tests and
  their bidirectional relationships are identical; archive integrity passes; required CI receives
  success; full product jobs are not selected.
- Scoped review: agent-authored implementation needs one reviewer attestation bound to the parent
  package/diff/delta/evidence digest; the scope approver cannot review, an explicit block prevents
  finalization, and every descendant/parent edge is inspected so change/revert history stales it;
  archive-only changes do not rerun or stale it; status explains how to request the check. Hosted
  routing additionally characterizes stale execution digests, self-review, and change/revert
  sequences, with explicit Git time/output/descendant/parent bounds.
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
- `REQ-change-013`, `REQ-change-016`: exact five-file post-verification persistence allowlist,
  every-parent review/finalization freshness, mixed-delivery rejection, change→revert rejection,
  and persisted reviewer-independence tests.
- `REQ-change-044`, `REQ-github-005`: exact finalization classifier, parent-check reuse,
  finalization-digest, archive integrity, post-merge binding, and release-wait tests.
- `REQ-change-045`: bounded snapshot query-count, deterministic graph, warm/cold parity, and
  one-pass owner-batch tests.
- `REQ-change-046`: missing/current/stale independent-review and review-only-child reuse tests.
- `REQ-change-029`: later valid sequence claims preserve earlier evidence, exact committed
  collision-owner ledgers remain exact for every named immutable member, and ordinary
  post-acceptance collision acknowledgements remain successor-covered.
- Rune PR #23: eight workflow-v2 changes each finalize in their originating branch, each
  finalization selects `archive_only=true`/`full=false`, and no active cleanup package remains.
- Incident regressions: direct renewal clears one-time adoption, adopted scope remains visible in
  status, cross-date post-move retries resume the unique archive, and local nested commands clear
  both `CI` and `GITHUB_ACTIONS`. The adoption-anchor regression uses a stable historical scope
  fixture rather than mutable active lifecycle files, so renewal and same-PR archival cannot
  silently change or remove its compile-time input. Committing the exact scoped-review artifacts
  after verification preserves current evidence, while mixing either review artifact with a source
  change fails closed.
- Adversarial regressions: missing adoption anchors fail closed; review blocks remain in an
  append-only trail, deleting and committing away a prior block still fails, and non-ASCII identity
  confusables are rejected; persisted attempts revalidate reviewer independence against the scope
  approval bound to their contract; native review/finalize mutations use the full every-parent
  verification validator so source change→revert history cannot regain fresh evidence; both
  initial-add and later-append review children use the hosted
  strict-prefix validator; unchanged block→pass recovery preserves the reviewed implementation
  ancestor, reuses only that ancestor’s implementation/trust checks, and makes the pass child the
  new successful review check; a fresh clone validates a terminal v2 archive after squash removes
  the implementation and intermediate block→pass objects; a journaled interruption after the first
  terminal write recovers;
  native/hosted review limits are read from one committed JSON document and workflow self-tests
  bind imports to the consuming archive-validation program; workflow-v2 origin removal,
  downgrade/revert history, later archive rewrites, torn journal envelopes, unreadable transaction
  backups, and PR-head trust-policy substitution all fail closed. Merged fork publication uses a
  base-controlled closed-PR event with object-only candidate access, while synthetic archive
  repositories prove unique introduction, unchanged successors, rewrite rejection, and
  shared-history-cap failure. The bootstrap verifier rejects arbitrary missing-guard identities,
  the guard protects all workflow/local-Action definitions, and a first-commit record with both
  v2 fields removed fails status, legacy accept/archive, and global check unless the same v1
  identity existed at the immutable pre-v2 cutoff.
- Trusted workflow bootstrap: PR #480 runs the full matrix and two independent reviews because its
  base predates the guard. The merged guard is then pinned by exact workflow SHA before backlog
  dogfooding; subsequent review/finalization/archive reuse requires its revision-bound check, while
  any protected policy-file edit blocks optimized reuse.
- Reviewer re-review regressions: workflow-v2 adoption selects and freezes the comparison-base
  cutoff, remains valid when pre-adoption branch commits are squash-collapsed, and reads genuine
  cutoff v1 records with omitted or explicit origin 1; the exact guard regex and `--no-renames`
  diff protect local-Action and root `action.yml` add/modify/delete/rename paths; the privileged
  post-merge checkout uses a full Action SHA; the PR #480 bootstrap requires the canonical baseline
  whose cutoff equals its frozen base; and workflow-origin history follows archive→reopen→rearchive
  identity across both canonical dated archive paths. Protected-path matching preserves NUL
  filename boundaries and catches a real workflow path containing an embedded newline. Archive
  subtree and workflow-v2 baseline rewrite→restore histories remain invalid because every bounded
  touching commit and parent is inspected.
- Private sandbox compatibility: the three colliding accepted legacy CHG-0001 workspaces remain
  readable with exact evidence and one explicit archive action each; discarded historical base
  objects do not leak raw Git fatal diagnostics into status output. After local workflow-v2
  adoption and a new sequence claim, bounded historical-ledger reconstruction keeps all three
  collision members exact while the draft reports one explicit next action.
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
