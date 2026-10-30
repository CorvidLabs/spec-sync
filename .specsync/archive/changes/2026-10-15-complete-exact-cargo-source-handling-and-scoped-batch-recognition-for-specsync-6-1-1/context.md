---
change: complete-exact-cargo-source-handling-and-scoped-batch-recognition-for-specsync-6-1-1
artifact: context
---

# Context

The original correction and Rustls patch are already accepted and archived. Exact-head review of f72f0b6 confirmed four separate residual gaps: scoped batch existing-file lookup disagrees with its writer; creation helpers ignore exact configured Cargo files; watch labels those existing files nonexistent; one testing row describes stale release versions. Independent read-only reproductions and flat/directory controls are preserved in the coordinator late-review reports. Leif directly approved the bounded follow-up proposal with "I approve". This records scope acceptance only.

Preserve prior accepted artifacts, user/contributor work, package identity, explicit mappings and exact-file confinement. No dependency changes, new flags, recursive project-root widening or validator ownership changes.
