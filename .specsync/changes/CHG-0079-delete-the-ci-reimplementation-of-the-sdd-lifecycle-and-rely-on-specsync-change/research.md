---
change: CHG-0079-delete-the-ci-reimplementation-of-the-sdd-lifecycle-and-rely-on-specsync-change
artifact: research
---

# Research

Every assertion made by the deleted machinery was enumerated before deletion and placed in exactly
one of two buckets: covered by SpecSync, or dropped deliberately.

## Covered by SpecSync

| Deleted assertion | Now proven by |
|---|---|
| Exactly one finalized archive per merged change | `specsync change audit --strict` |
| Archive evidence matches the accepted change state | `specsync change audit --strict` |
| Living SDD policy and spec coherence | `specsync change audit --strict` |
| Lifecycle state machine behavior | `cargo test` over `src/change.rs` |

## Dropped deliberately

| Deleted assertion | Why it is safe to drop |
|---|---|
| Archive-only finalization is a single-parent child commit | Only meaningful under the separate-tip constraint being removed |
| The child commit is the exact approved change-to-archive move | Same |
| Trusted-policy ancestry is reusable across the parent chain | Same |
| Required checks share one product ancestor and CI run | Same; CI now verifies the pull request as a whole |
| The archive subtree did not change after its unique introduction | Intra-branch mutation is visible in the reviewed net diff |
| The archive path does not already exist in an introduction parent | Same |
| Unauthorized edits to protected paths are rejected | Moves to CODEOWNERS plus required code-owner review, which is strictly stronger than a guard that never blocked a merge |

## Product dependencies confirmed before deletion

`src/change.rs:329` reads `.github/scripts/lifecycle-validation-limits.json`, so that file is
retained. `src/change.rs:2493` names `.github/workflows/post-merge-archive.yml` in a default
strict-path list; that entry is a path pattern rather than a file read, so it becomes inert rather
than broken, and it is left for a product-scoped change.
