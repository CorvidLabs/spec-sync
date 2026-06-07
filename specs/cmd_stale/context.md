---
spec: cmd_stale.spec.md
---

## Key Decisions

- **Git history as source of truth**: Staleness is based on commits to source files since the spec's last commit, not file modification time.
- **Most stale first**: Results sort by maximum commits behind so the highest-risk specs appear first.
- **Skip untracked history**: Files with no usable git history are skipped to avoid false failures during bootstrap.

## Files to Read First

- `src/commands/stale.rs` — CLI implementation and formatting.
- `src/git_utils.rs` — Commit hash and commit distance helpers.

## Current Status

Stable. The command is implemented and integrated with common output formats.

## Notes

- Use `specsync report` when stale information should be combined with coverage and validation status.
