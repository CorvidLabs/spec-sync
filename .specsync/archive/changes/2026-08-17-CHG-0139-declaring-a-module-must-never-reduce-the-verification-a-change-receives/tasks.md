---
change: CHG-0139-declaring-a-module-must-never-reduce-the-verification-a-change-receives
artifact: tasks
---

# Tasks

- [x] Reproduce on live work, not only a fixture: two real changes whose declared scopes differ,
      with the narrower one receiving more verification.
- [x] Bisect the trigger to a single condition, ruling out test paths, caching, worktree identity,
      `--spec` count, and a prior failed check.
- [x] Confirm the mechanism at the line level in `verification_commands_for_change`.
- [x] Rule out a deferred full run at any later lifecycle stage.
- [x] Measure the historical spread across the archive.
- [x] Decide scope: fix monotonicity; leave zero-match out because `REQ-change-058` forbids the
      output capture it needs. State that rather than omitting it silently.
- [x] Create the change record BEFORE writing code.
- [x] Implement the per-module condition.
- [x] Assert the property as a superset relation, not by example.
- [x] Add the vacuity control that keeps targeted verification alive.
- [x] Prove discrimination against a separate checkout with `src/change.rs` provably unmodified,
      checking the unfixed build's exit code before trusting its result.
- [x] Confirm the vacuity control passes on BOTH binaries.
- [x] `cargo test`, `cargo clippy -- -D warnings`, `cargo fmt --check`.
- [x] Count `#[test]` markers before and after.
- [x] Whole-board check: no drill may change state, since no drill covers this surface.
      Measured: pass=45 fail=10 skip=0 total=55, identical to the pre-change board. Zero drills moved.
- [x] CHANGELOG entry disclosing the slowdown and the unrepaired archive records.
