---
spec: cmd_lifecycle.spec.md
---

## Key Decisions

- **Frontmatter-only mutation**: Status updates replace only the `status:` line inside YAML frontmatter to avoid accidental body edits.
- **Guarded transitions by default**: Invalid lifecycle jumps and failed guards require `--force`, keeping CI and human workflows predictable.
- **Machine-readable output**: Status, guard, auto-promote, and enforce paths honor JSON output for automation.

## Files to Read First

- `src/commands/lifecycle.rs` — Lifecycle command implementation and unit tests.
- `src/types.rs` — `SpecStatus`, lifecycle config, and transition guard types.

## Current Status

Stable. Core status transitions, guard evaluation, auto-promotion, and enforcement are implemented.

## Notes

- Keep lifecycle commands side-effect-light in dry-run/guard paths; only promote/demote/set/auto-promote without dry-run should edit specs.
