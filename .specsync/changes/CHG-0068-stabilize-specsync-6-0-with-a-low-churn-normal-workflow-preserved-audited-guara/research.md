---
change: CHG-0068-stabilize-specsync-6-0-with-a-low-churn-normal-workflow-preserved-audited-guara
artifact: research
---

# Research

## Evidence

- PR #455 passes implementation, unit, integration, and coverage checks; `spec-check` fails because
  no active lifecycle workspace covers its changed schema paths. The required gate only propagates
  that failure.
- CorvidLabs/rune PR #23 is a valid 5.2 post-merge cleanup for eight changes, but it modifies 103
  archive files and repeats review/CI despite changing no runtime or canonical contract.
- PR #471 has fresh successful verification and is intentionally frozen as a separate history track.
- PR #462 is a conflicting monolithic lifecycle draft and is excluded.

## Compatibility conclusions

- Existing persisted two-approval records remain byte-compatible historical evidence.
- New 6.0 records use one approval and the same active/archive paths for every change.
- Strict is a validator set selected by `--strict`, policy, or release/security classification; it
  is not persisted as a lifecycle mode and cannot fork transitions or layout.
- `change finalize` mutates lifecycle and canonical files but never calls a GitHub merge API.
- The archive-only lane is a positive validator, not a broad path-based test skip.
- Parent required-check and scoped-review evidence is bound to the implementation commit so the
  metadata-only child does not repeat expensive work.

## Threat model

The finalizer must reject stale approval, failed or missing parent checks, missing scoped review,
unapproved paths, delivery-tree changes, digest mismatch, replay under another PR, invalid archive
ownership, and release attempts with missing merge binding. Pull-request-controlled input cannot
claim green parent checks or expand the archive-only allowlist.
