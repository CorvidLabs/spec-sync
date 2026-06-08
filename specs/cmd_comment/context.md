---
spec: cmd_comment.spec.md
---

## Key Decisions

- Reuses the `check` pipeline verbatim: `load_and_discover` → `run_validation` (collect mode) → `compute_coverage` → `compute_exit_code`. This guarantees the comment's pass/fail badge matches what `check` would exit with in CI.
- `render_check_comment` (in `src/comment.rs`) is the single renderer for the markdown body. There is intentionally no second comment-generation path — both the marketplace action and the project CI shell out to `specsync comment`.
- Posting vs printing is the only branch in this wrapper: with `--pr N` it resolves the repo and shells out to `gh pr comment`; without `--pr` it prints the body to stdout.
- Enforcement resolution: `--enforcement` overrides config; `--strict` implies strict enforcement; otherwise `config.enforcement` is used.

## Files to Read First

- `src/commands/comment.rs` — the command wrapper (this module)
- `src/comment.rs` — `render_check_comment`, `detect_branch`, and the suggestion/grouping helpers
- `src/commands/mod.rs` — `run_validation`, `compute_exit_code`, `load_and_discover`, `build_schema_columns`
- `src/github.rs` — `detect_repo`, `resolve_repo`

## Current Status

Implemented and stable. The `comment` renderer is extensively unit-tested; the wrapper itself (validation reuse + gh shell-out branch) has no inline tests.

## Notes

- The `_base` parameter is currently unused (prefixed with `_`); it is reserved for diff-base wiring.
- `gh pr comment` failures and missing `gh` are handled explicitly with distinct error messages and `process::exit(1)`.
