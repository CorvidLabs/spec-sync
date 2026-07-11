---
change: CHG-0009-make-accepted-evidence-squash-safe-and-harden-the-5-0-release-path
artifact: context
---

# Context

PR #335 passed its pull-request matrix but was squash-merged as `884ad33`. Main then failed only `spec-check`
because six accepted records referenced verification commits no longer in main history. The accepted scoped content,
contracts, and approval digests remain unchanged. No 5.0 tag or crate publication has occurred.

The implementation keeps ancestry as the fast path and permits a squash fallback only when the accepted workspace is
unchanged at a remote default ref containing HEAD. Historical accepted records are archived. Release automation now
separates exact `v5.0.0` publication from the floating `v5` Action alias.

PR #340's first matrix exposed a Windows-only fixture dependency on global `core.autocrlf`. The failure also prompted
an adversarial digest audit, which found ambiguous unframed path/content concatenation and missing file-mode evidence.
The failed acceptance was reversed before further implementation. The digest format is being hardened before 5.0
because no public 5.0 evidence schema has shipped.
