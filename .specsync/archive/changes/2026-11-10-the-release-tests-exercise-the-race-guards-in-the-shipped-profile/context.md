---
change: the-release-tests-exercise-the-race-guards-in-the-shipped-profile
artifact: context
---

# Context

Issue #614 (p1). After #581 gated seven race tests to debug builds, three TOCTOU guards had
no test that could run under `cargo test --release`, and no pipeline runs release tests.
Leif approved CI-11 on 2026-11-06 ("When I run the release tests, the race guards in the
shipped profile are tested too"). It is recorded in `hi/ci.md` on this branch.

Each guard now has an in-crate unit test that retains the root, replaces the directory
under the same path, and asserts the guard refuses. The test controls the timing, so it
needs no debug-only rendezvous and compiles in every profile. The release-candidate lane
gains `test-release-guards` (`cargo test --release --locked --bin specsync --
release_race_guard`).

Not approved by the worker: this lands hi intent (CI-11) and changes the release-candidate
lane, so under Leif's 2026-11-10 merge rule it stops at a ready PR for his scope approval.
