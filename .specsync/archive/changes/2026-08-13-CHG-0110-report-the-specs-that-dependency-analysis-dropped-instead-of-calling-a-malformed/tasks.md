---
change: CHG-0110-report-the-specs-that-dependency-analysis-dropped-instead-of-calling-a-malformed
artifact: tasks
---

# Tasks

- [x] Report parser frontmatter errors instead of discarding them
- [x] Report a spec whose frontmatter cannot be parsed as dropped
- [x] Report a spec declaring no `module` as dropped
- [x] Reuse the validator's wording so `check` and `deps` agree on identical input
- [x] Confirm a well-formed project still reports a valid graph and exits zero
- [x] `cargo fmt --all -- --check` clean
- [x] `cargo clippy -- -D warnings` exit 0
- [x] `cargo test` green — 2210 unit, 331 integration, 0 failures
