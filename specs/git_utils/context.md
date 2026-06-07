---
spec: git_utils.spec.md
---

## Key Decisions

- **Git CLI wrapper**: The module shells out to `git` rather than linking a git library, keeping dependencies small.
- **Safe defaults**: Git failures return `None`, `0`, or `false` so callers can decide whether to warn, skip, or fail.
- **Repository-root execution**: Every command runs with `current_dir(root)` to preserve expected path resolution.

## Files to Read First

- `src/git_utils.rs` — Shared git wrappers.
- `src/commands/stale.rs` and `src/commands/report.rs` — Main callers.

## Current Status

Stable. Used by staleness detection, reports, and spec scoring freshness.

## Notes

- Do not make these helpers panic on git errors; graceful degradation is part of the public behavior.
