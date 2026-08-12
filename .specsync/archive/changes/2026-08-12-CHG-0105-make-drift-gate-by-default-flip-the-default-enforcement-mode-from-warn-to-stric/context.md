---
change: CHG-0105-make-drift-gate-by-default-flip-the-default-enforcement-mode-from-warn-to-stric
artifact: context
---

# Context

Step 1 severed `specsync check` from the trust layer, so lifecycle state no longer sets the
exit code. That was correct, but it left the product unable to fail by default: the trust
gate had been the only unconditional gate in `check`.

`EnforcementMode::Warn` is `#[default]` and documented "Report violations but always exit 0
(default, non-blocking)". So before this change:

| | non-strict | `--strict` |
|---|---|---|
| spec drift (the product) | exit **0** | exit 1 |
| SDD/trust errors | exit **1** | exit 1 |

A repository with a deleted source file and undocumented exports passed `specsync check`,
while a lifecycle bookkeeping problem failed it. The tool's own reason for existing did not
gate; the machinery around it did.

This change flips the default so drift gates, closing the hole Step 1 opened. It ships
separately from Step 1 on purpose: both are user-visible exit-code changes, and combining
them would make the two indistinguishable under `git bisect`.

## What it also fixes, unintentionally

The `draft_spec_check_reports_skipped_validation` fixture in this repository's own test suite
carries a real error the warn default was swallowing:

```
✗ Source file has duplicate spec ownership: src/auth/service.ts
  (also mapped by specs/draft-mod/draft-mod.spec.md)
```

That is issue #508 — "duplicate spec ownership is only detectable in CI, never locally."
Under a strict default it gates locally. That spec-sync's own fixtures were quietly carrying
the error is a reasonable argument that `warn` was the wrong default.
