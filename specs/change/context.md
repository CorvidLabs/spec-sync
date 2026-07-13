---
spec: change.spec.md
---

# Context

Canonical module maturity remains under `specsync lifecycle`; SDD delivery uses six separate states. `.specsync/sdd.json` is a dedicated versioned policy so existing projects remain opt-in. Human artifacts and deltas are Markdown, while state, approvals, and evidence are JSON. Verification commands come only from policy and run without a shell.

The public lifecycle remains one module for 5.0 to avoid a late high-risk refactor. Its intended internal seams are state/transitions, approvals/evidence, semantic deltas, Git/path coverage, effective-contract validation, and adoption/import. Extract those seams after 5.0 without changing the public API. Release evidence is recorded in accepted/archived change workspaces and the PR matrix rather than frozen as a permanent claim here.

`check_project_quiet` shares all fail-closed validation with `check_project` but discards configured child-command output so `specsync comment` can emit a single bounded markdown protocol. Explicit verification and ordinary checks retain streamed diagnostics.

Accepted review fixes use `change reopen`, which transitions only stale governed delivery evidence to `verifying`. The approval ledger appends a versioned reopen event containing the untouched prior verification and superseded closing approval. `canonical_applied` distinguishes re-verification from initial delivery so fresh acceptance cannot apply the semantic delta twice; it is lifecycle-only state and is excluded from definition approval digests.

For schema-v1 compatibility, false `canonical_applied` values are omitted from new persisted JSON. Definition-evidence validation recognizes both the original omitted encoding and the transitional explicit-false encoding, preserving approvals and verification created on either side of the field's introduction; true values remain durable for reopened and accepted workspaces.
