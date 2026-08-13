---
change: CHG-0113-staleness-detection-must-refuse-a-repository-with-no-commits-instead-of-reportin
artifact: tasks
---

# Tasks

- [x] Add `has_commits` to `git_utils`, implemented via `rev-parse --verify HEAD`
- [x] Guard staleness on both preconditions
- [x] Carry the reason through so the two causes stay distinguishable
- [x] Report the reason in the machine-readable payload as well as the text output
- [x] Confirm a repository with one commit is unaffected
- [x] `cargo fmt --all -- --check` clean
- [x] `cargo clippy -- -D warnings` exit 0
- [x] `cargo test` green — 2210 unit, 331 integration, 0 failures
