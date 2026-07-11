---
change: CHG-0005-close-final-fail-closed-review-gaps-in-5-0-lifecycle-evidence-and-pr-reporting
artifact: context
---

# Context

The final automated review of PR #335 found six fail-closed gaps after CHG-0004 was accepted. Persisted change state can currently supply unsafe IDs or spec scopes, historical tombstone parsing can silently discard corruption, CI can rerun commands without requiring the recorded verification evidence, corrupt approval ledgers can be replaced, and PR comments omit SDD-only failures.

This follow-up stays on the existing branch and PR. The principal files are `src/change.rs` and `src/commands/comment.rs`; canonical behavior is tracked by the `change` and `cmd_comment` specs.
