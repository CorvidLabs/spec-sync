---
change: CHG-0140-a-stale-sequence-ledger-must-not-be-committed-backwards
artifact: plan
---

# Plan

1. Reproduce through the real CLI, not by reasoning about the code: construct HEAD=3 /
   worktree=1 and run `change check --commit`.
2. Confirm the reproduction matches the issue's live evidence rather than merely resembling it.
3. Enumerate every caller of `git_commit_all` before changing any of them.
4. Put the floor at the shared staging point so a later commit path cannot bypass it.
5. Raise rather than refuse, and return the pair so the caller can disclose.
6. Write the control first: a working tree ahead of the mark must be untouched.
7. Prove discrimination through the CLI on two genuinely different binaries.
8. Check for a paired pin drill BEFORE shipping, by reading drills rather than grepping for
   the issue number — 037 never cites #533 and a number grep misses it.
9. Invert 037, and verify the inversion fails on an origin/main binary.
10. Whole-board check: exactly one gate may change state.
11. Full lifecycle with semantic deltas; do not hand-edit `specs/`.
