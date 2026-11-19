---
change: an-archive-introduction-listing-over-the-output-bound-is-split-instead-of-read-one-commit-at-a-time
artifact: context
---

# Context

#942, a Codex P2 on #918, deferred by Leif's call. `archive_introduction_listings`
(`src/change/read_cache.rs`) makes one `diff-tree --stdin` call for every uncached commit. When
the combined output passed the 64 MiB bound, `run_git_bounded` failed and `.ok()?` returned `None`,
so the caller read every commit with its own `diff-tree`. That is the per-commit process growth
REQ-change-135 removed. Verdicts were unchanged; only speed.

Now an over-bound part is split in halves until each fits. A single commit over the bound is left
out, so the caller reads it on its own, as before. Any other Git failure still returns `None`. The
red commit adds `archive_introduction_listings_bounded`, the same function with the bound as a
parameter, so a test can lower it.

Chough: spec-sync's `.chough/` is still 0.5.x until spec-sync#1073 merges; this change goes through
a Chough 0.6.0 session before review (Leif's rule, relayed by t370 on 2026-11-13).
