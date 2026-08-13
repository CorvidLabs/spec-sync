---
change: CHG-0110-stop-printing-green-result-lines-for-checks-that-could-not-run-when-frontmatter
artifact: tasks
---

# Tasks

- [x] Derive `frontmatter_invalid` from the frontmatter error list already collected
- [x] Report the source-file check as skipped when frontmatter is invalid
- [x] Report the required-section check as skipped, after the existing draft branch
- [x] Report the dependency check as skipped
- [x] Reuse the `⊘ … skipped` wording the draft path already established
- [x] Confirm the exit status is unchanged
- [x] `cargo fmt --all -- --check` clean
- [x] `cargo clippy -- -D warnings` exit 0
- [x] `cargo test` green — 2210 unit, 331 integration, 0 failures
