---
change: CHG-0068-stabilize-specsync-6-0-with-a-low-churn-normal-workflow-preserved-audited-guara
artifact: design
---

# Design

## One lifecycle

Every change uses `.specsync/changes/<id>/` while active and the existing
`.specsync/archive/changes/YYYY-MM-DD-<id>/` destination when finalized. `approvals.json` contains
one current human scope approval for a 6.0 change. Existing two-approval records remain readable and
verifiable as historical evidence, but new work has no closing approval or lifecycle-mode field.

The newcomer path is fixed and visible:

`change new` → `change approve` → implement → `change check` → ordinary PR review →
`change finalize` → GitHub merge.

`change status` emits exactly one next action for every state. SpecSync exposes no merge command and
never bypasses or duplicates GitHub branch protection.

## Same-PR finalization

`change finalize` verifies the current approval, implementation evidence, completed tasks,
bidirectional code/spec ownership, semantic deltas, and any required automated validators. It
applies canonical deltas, records `accepted-state.json`, rewrites the package as terminal evidence,
and moves the same package to the dated archive in one transaction. Automation commits only these
approved lifecycle/archive changes to the existing PR.

The archive-only CI lane safely classifies the finalization diff. It requires:

- every required check on the parent implementation commit to be green;
- only the exact approved lifecycle/archive paths to differ from that parent;
- identical code, canonical spec, requirements, tests, configuration, and other delivery-tree
  relationships;
- valid archived evidence, bidirectional ownership, and finalization digest; and
- a successful result reported into the required CI gate.

It does not rerun the product test matrix. A metadata-only post-merge integrity job may bind the
actual merge SHA/tree to the already-merged archive digest, but it writes no code files and creates
no user checkpoint or PR.

## Scoped independent review

Agent-authored implementation commits require one independent review before finalization. The
review input is bounded to the change package, implementation diff, canonical semantic delta, and
targeted evidence. Its attestation binds the implementation parent commit and is reused by the
archive-only lane; finalization metadata does not trigger the reviewer again. Status explains that
the review is needed and tells the user to open or update the PR so the configured
`SpecSync scoped review` check can run.

## Strict validator policy

The existing global `--strict` flag, project policy, or deterministic release/security
classification can require full history, the full product suite, authentication/security analysis,
or release validation. These are additional validators recorded in the same `verification.json`.
Strict preserves the powerful historical and trust checks without creating a mode: it does not
change the state machine, commands, approval count, change package, review step, finalization,
archive path, or GitHub merge flow.

## Bounded validation and retrieval

One invocation-scoped snapshot memoizes active/archive records, canonical owners, Git comparison
bases, candidate entries, parent-check evidence, and terminal evidence. Graphs are ordered
deterministically and owner batches are validated once. Canonical contracts contain durable
behavior and decisions only; transient logs are reduced to concise evidence. Retrieval selects the
current contract and relevant decision summaries for affected components.

## Stabilization carry-forward

Only reviewed source/test portions of PRs #455, #456, #457, and #449 are carried forward. Their
canonical requirements are integrated through this change. PR #463 contributes bounded lifecycle
query improvements only. PRs #471 and #462 remain separate.
