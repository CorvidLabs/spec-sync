---
change: CHG-0104-sever-specsync-check-and-comment-from-the-trust-layer-lifecycle-state-becomes-i
artifact: research
---

# Research

- `src/commands/check.rs` — `audit_project` call, error branch, `process::exit(1)`,
  and the existing `✓ SDD lifecycle valid (N active change(s))` line that this
  change reduces to informational output.
- `src/types.rs` — `EnforcementMode::Warn` is `#[default]`, documented "Report
  violations but always exit 0 (default, non-blocking)".
- `src/commands/mod.rs` — `compute_exit_code`: `Warn` returns 0 unconditionally;
  `Strict` returns 1 on any error and on warnings when `--strict`.
- `src/commands/comment.rs` — `check_project_quiet` results merged into the
  reported error/warning sets.
- Sandbox drill 038 was written before this change specifically to pin the drift
  behaviour that must survive it; it passes on both 6.0.0 and 5.2.0.
