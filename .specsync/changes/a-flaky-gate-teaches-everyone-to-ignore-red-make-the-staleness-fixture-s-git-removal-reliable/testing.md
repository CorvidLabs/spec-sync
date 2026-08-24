---
change: a-flaky-gate-teaches-everyone-to-ignore-red-make-the-staleness-fixture-s-git-removal-reliable
artifact: testing
---

# Testing

`cargo test --release --test integration staleness_unmeasurable` — 12 passed, run three times.

Honest limitation: these tests **always passed locally**, including before this change, so local
runs cannot demonstrate the fix. The flake is CI-specific — Linux, slower disk, higher
parallelism. The proof is CI staying green across subsequent runs, and this change is written so
that if the flake recurs, the retry has already excluded "transient concurrent writer" and the
next investigator starts from a smaller hypothesis space.
