---
change: CHG-0104-sever-specsync-check-and-comment-from-the-trust-layer-lifecycle-state-becomes-i
artifact: plan
---

# Plan

1. Replace the `audit_project` error/exit branch in `src/commands/check.rs` with an
   informational summary that never sets a non-zero exit code, preserving the JSON
   shape's `sdd` field for machine consumers.
2. Keep shape warnings (unparseable state, illegal state) on stderr in text mode.
3. Remove the SDD error/warning merge in `src/commands/comment.rs` so PR comments
   report spec results only.
4. Record the user-visible exit-code change in `CHANGELOG.md`.
5. Add the semantic deltas for `cmd_check` and `cmd_comment`.
6. Verify with the component command for `cmd_check`; run the sandbox drills that
   pin the product (038) and the happy path (028).
