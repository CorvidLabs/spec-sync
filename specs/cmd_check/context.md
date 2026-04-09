---
spec: cmd_check.spec.md
---

## Key Decisions

- Module follows the standard command pattern: load config, discover specs, delegate to library module, format output, exit
- Spec was created during the 100% coverage push to dogfood spec-sync on its own codebase

## Files to Read First

- `src/commands/check.rs` — primary source file

## Current Status

Fully implemented and stable. Spec created to achieve 100% file coverage.

## Notes

- This module is part of the command layer — it orchestrates library modules rather than containing domain logic
