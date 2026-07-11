---
change: CHG-0009-make-accepted-evidence-squash-safe-and-harden-the-5-0-release-path
artifact: context
---

# Context

PR #335 passed its pull-request matrix but was squash-merged as `884ad33`. Main then failed only `spec-check`
because six accepted records referenced verification commits no longer in main history. The accepted scoped content,
contracts, and approval digests remain unchanged. No 5.0 tag or crate publication has occurred.
