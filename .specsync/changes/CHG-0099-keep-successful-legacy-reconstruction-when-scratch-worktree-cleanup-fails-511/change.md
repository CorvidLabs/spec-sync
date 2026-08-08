---
id: CHG-0099-keep-successful-legacy-reconstruction-when-scratch-worktree-cleanup-fails-511
state: implementing
type: bug_fix
base_commit: 2b43f39ff73ed624c54996913b28ac698f60f9c3
---

# Keep successful legacy reconstruction when scratch worktree cleanup fails (#511)

## Intent

Keep successful legacy reconstruction when scratch worktree cleanup fails (#511)

## Affected Canonical Specs

- `change`

## Acceptance Criteria

- reconstruct_legacy_at_anchor returns Ok reconstruction even when git worktree remove fails; best-effort prune+rm cleanup; regression covered by legacy_reconstruction_deduplicates_identical_transitions_but_rejects_distinct_evidence with forced remove failure

## No-spec Rationale

No Public API or requirements change; best-effort cleanup of disposable worktree only. Living change.spec.md already documents reconstruct paths; behavior fix is internal fail-open on hygiene.
