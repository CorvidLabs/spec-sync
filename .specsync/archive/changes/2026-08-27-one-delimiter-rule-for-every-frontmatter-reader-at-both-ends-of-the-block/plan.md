---
change: one-delimiter-rule-for-every-frontmatter-reader-at-both-ends-of-the-block
artifact: plan
---

# Plan

1. Read #716, #715, #699 and #705, then re-derive from source rather than from the report. Build a
   binary from a SEPARATE checkout of unfixed `main` and measure every claim against it.
2. Sweep for every frontmatter/delimiter reader in the repository before touching one, and record
   the verdict for each — fixed, or left with a stated reason.
3. Add `is_frontmatter_delimiter` and `split_frontmatter` to `src/parser.rs`; route
   `strip_frontmatter` and `parse_checked_issue_references` through them and widen
   `FRONTMATTER_RE`'s delimiter classes to match.
4. Write the tests, each labelled CONTROL, DISCRIMINATOR or CHARACTERIZATION, and run them against
   the control binary. Every DISCRIMINATOR must fail there; every CONTROL and CHARACTERIZATION must
   pass there, and be labelled as such rather than counted as evidence.
5. Pin the two behaviours #715 changed without stating: the BOM-prefixed empty artifact and the
   LF body for a CRLF-only body. Both are CHARACTERIZATION — they pass on the control, which is
   the point.
6. Write the semantic deltas for `parser` and `change`, and update the companions.
7. `cargo fmt`, CI-equivalent `cargo clippy -- -D warnings`, full `cargo test`, `specsync check
   --strict --require-coverage 100`, `specsync change check`, `specsync change audit --strict`.
8. Open the PR. Do not merge.
