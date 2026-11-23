---
change: download-the-history-objects-a-partial-clone-lacks-in-one-fetch-with-the-user-s-own-git-before-specsync-reads-them
artifact: plan
---

# Plan

## Approach

One helper in `read_cache.rs`, `ensure_partial_clone_objects`, run once per process before the first lifecycle-history read (`scoped_review_file_at_commit`, `scoped_review_git_text`). In a partial clone it lists, isolated, every missing object under `.specsync` and the canonical requirements paths reachable from `HEAD` and the default branch, and downloads them in one fetch with the user's own Git.

## Out of scope

A full clone (no promisor remote) changes nothing. Shallow clones are #1072. Reads that are not history (`git_utils`) are not touched.

## Steps

1. Failing tests through the binary: a blobless clone whose remote answers only with the user's configuration.
2. The helper: promisor detection, the isolated listing, the one bounded fetch with no terminal, the stderr line.
3. The refusal when the download fails, naming one `git fetch … <oid> …`.
4. Docs, CHANGELOG, spec invariant.

## Risks

SpecSync runs the user's credential helper and URL rewrites for this one fetch: the new trust is the helper, not the destination (the repository's own promisor remote). The fetch reaches the network during commands people think of as read-only, only in a partial clone.

## Constraints consulted

Chough's 120 s bound on each SpecSync call (`CHOUGH_NATIVE_TIMEOUT_SECONDS`), hence 60 s for the fetch; the lifecycle spawn ceilings (a full clone adds at most the one promisor query).
