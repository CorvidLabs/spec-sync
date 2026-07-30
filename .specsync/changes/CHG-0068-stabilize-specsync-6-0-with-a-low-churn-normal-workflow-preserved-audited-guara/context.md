---
change: CHG-0068-stabilize-specsync-6-0-with-a-low-churn-normal-workflow-preserved-audited-guara
artifact: context
---

# Context

SpecSync 5.2 requires definition and closing approvals and usually repeats the full product matrix
for lifecycle-only follow-up commits. The current post-merge archive model adds a second code PR
even when no product files change. CorvidLabs/rune PR #23 is the compatibility case: archiving eight
already-accepted changes touches 103 files and repeats review and CI.

The open PR backlog also shares a strict-check failure mode: implementation tests pass, but old
branches lack an active change whose scope covers their meaningful paths. Large lifecycle branches
then amplify the problem by repeatedly scanning Git history and accepted evidence.

SpecSync 6.0 has one workflow and one layout for every change. A human approves scope once. Code,
canonical specs, requirements, and tests are implemented and checked bidirectionally. Agent-authored
work receives one independent scoped review. `change finalize` then creates a metadata/archive-only
commit on the same PR, and GitHub remains responsible for merge protections and the merge itself.

The archive-only commit must not pay for the product matrix again. Its lane proves that the parent
implementation commit already has the required green checks, the finalization diff contains only
approved lifecycle/archive paths, code/spec/tree relationships are unchanged, and archive ownership
and integrity remain valid. Explicit `--strict`, project policy, or release/security classification
adds the existing powerful full-history, full-suite, security, and release validators to the same
evidence and layout. Strict validation never changes approvals, state transitions, commands,
finalization, archival, or the newcomer workflow.

The approval-boundary regression discovered during implementation is resolved inside this same
workflow: the original human event binds the stable intent and affected scope, while a separate
execution digest binds artifacts, deltas, tests, canonical materialization, and lifecycle metadata.
The post-approval CHG-0068 edits were classified in `approvals.json`; none expands its ten approved
criteria or declared affected specs/paths, so the original approval remains valid and automated
verification plus scoped review—not another human prompt—must absorb those edits.
