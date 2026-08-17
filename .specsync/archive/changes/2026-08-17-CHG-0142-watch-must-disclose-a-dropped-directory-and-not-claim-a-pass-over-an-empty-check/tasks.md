---
change: CHG-0142-watch-must-disclose-a-dropped-directory-and-not-claim-a-pass-over-an-empty-check
artifact: tasks
---

# Tasks

- [x] Reproduce both halves: a dropped directory that is never mentioned, and a
      green `All checks passed!` over a run that examined no specs.
- [x] Resolve the watch set into watched and skipped rather than filtering, so
      the dropped paths survive to be reported.
- [x] Report each skipped path with its role on stderr, in both output modes.
- [x] Keep a missing directory non-fatal, and keep the empty set fatal.
- [x] Stop inferring success from the check child's zero exit: claim a pass only
      on positive evidence that specs were examined.
- [x] Add the vacuity control — a real, passing spec set must still report
      `All checks passed!` — and confirm it passes on BOTH binaries.
- [x] Confirm the two disclosure tests FAIL on an unfixed binary built from a
      separate checkout.
- [x] `cargo test`, `cargo clippy -- -D warnings`, `cargo fmt --check`.
- [x] Confirm drill 060 flips and the whole board moves by exactly one.
