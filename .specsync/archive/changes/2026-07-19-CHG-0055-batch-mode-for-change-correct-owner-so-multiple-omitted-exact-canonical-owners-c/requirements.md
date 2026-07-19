---
change: CHG-0055-batch-mode-for-change-correct-owner-so-multiple-omitted-exact-canonical-owners-c
artifact: requirements
---

# Requirements

### REQ-change-038

The verified lifecycle SHALL allow one transactional batch of audited exact acceptance-owner
corrections so rollout-era gaps with many omitted owners need only one reapprove → verify → accept
cycle, without weakening per-entry scope, ownership, or append-only sequencing rules.

Acceptance Criteria

- A batch may be supplied as repeated path/module pairs, a manifest file, or `--all-missing` with
  one canonical module.
- Every entry is validated independently against the same rules as a single `correct-owner`.
- Each accepted entry becomes its own sequenced `AcceptanceOwnerCorrection` record.
- If any entry is invalid, the command fails closed and persists no corrections from the batch.
- Single-path `correct-owner` remains supported and equivalent to a one-entry batch.

### REQ-cli-args-006

The shared CLI grammar SHALL expose batch selection for `change correct-owner` while keeping actor
and reason mandatory and rejecting empty or conflicting selection modes before domain mutation.

### REQ-cmd-change-004

The change command adapter SHALL resolve batch correct-owner selection, delegate policy to the
change domain, and render text/JSON results without partial lifecycle mutation on failure.
