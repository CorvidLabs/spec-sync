---
change: one-delimiter-rule-for-every-frontmatter-reader-at-both-ends-of-the-block
artifact: tasks
---

# Tasks

- [x] Build a control binary from a separate checkout of unfixed `main` (`d6f266a4`) and measure
      every claim in #716 against it.
- [x] Sweep for every frontmatter/delimiter reader and record fixed-or-left for each.
- [x] Add `is_frontmatter_delimiter` and `split_frontmatter` in `src/parser.rs`.
- [x] Route `strip_frontmatter` through the shared scan.
- [x] Route `parse_checked_issue_references` through the shared scan, preserving its empty-block
      verdicts and the exact YAML bytes it hands to `serde_saphyr`.
- [x] Widen `FRONTMATTER_RE`'s delimiter classes to `[ \t\r]*` and cross-reference both spellings.
- [x] Seven DISCRIMINATOR assertions, each shown RED against the control binary.
- [x] Two CONTROL assertions (`----`/`--- x`/indented `---` still refused; checked-refs empty-block
      verdicts unchanged) — labelled, and NOT counted as evidence.
- [x] Three CHARACTERIZATION assertions (BOM-prefixed empty artifact; LF body for a CRLF-only body;
      the four-dash residual) — labelled, and they pass on the control, which is the point.
- [x] Semantic deltas for `parser` (REQ-parser-003 plus Invariants/Behavioral Examples/Error Cases)
      and `change` (invariant 35).
- [x] Companions: `specs/parser/context.md`, `tasks.md`, `testing.md`, `specs/change/context.md`,
      `testing.md`.
- [x] Record the `commands/lifecycle.rs` sibling in `specs/parser/tasks.md`, since #715's claim that
      it was named in `tasks.md` does not hold in the living tree.
- [x] `cargo fmt --check`, `cargo clippy -- -D warnings`, `cargo test` (2407 unit + 407
      integration), `specsync check --strict --require-coverage 100`.
