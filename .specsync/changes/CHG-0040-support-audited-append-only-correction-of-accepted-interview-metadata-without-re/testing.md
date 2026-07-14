---
change: CHG-0040-support-audited-append-only-correction-of-accepted-interview-metadata-without-re
artifact: testing
---

# Testing

## Domain unit coverage

- Correct `public_contract` and `architecture_risk` in both directions from an accepted workspace.
- Reject empty actor/reason, unsupported fields, non-boolean values, unchanged values, draft through
  verifying states, and archived workspaces without changing any lifecycle file.
- Prove original answers, selected artifacts, definition/closing approvals, and prior verification
  remain unchanged and inspectable after correction.
- Prove `no -> yes` adds the exact deterministic artifacts, never removes artifacts, writes templates
  atomically, and keeps `canonical_applied` true.
- Reject missing, malformed, unsupported-version, truncated, reordered, value-chain-mismatched, and
  digest-tampered correction ledgers.
- Prove correction metadata and definition digests are identical across different checkout roots and
  separator conventions.
- Prove correction succeeds after squash integration using recorded canonical acceptance evidence and
  rejects unintegrated or fabricated accepted history.
- Reaccept one correction, perform a second correction, and prove the complete ordered chain remains
  valid while a correction attempted before reacceptance is rejected.
- Prove fresh verification and closing approval are mandatory and acceptance produces no second
  canonical version bump or changelog application.

## CLI integration coverage

- Parse and execute `change correct` with explicit field, value, actor, and reason.
- Assert deterministic JSON contains the original/effective view, correction event, prior evidence,
  added artifacts, approval health, and next action.
- Assert text correct/show/status output explains the correction and next gate.
- Assert Clap and domain failures exit non-zero with specific diagnostics and no mutation.

## Release regression

- Run the complete unit and integration suites, `specsync check --strict`, spec scoring, the Astro
  documentation build, dependency audit, and `fledge trust verify`.
