---
change: CHG-0105-make-drift-gate-by-default-flip-the-default-enforcement-mode-from-warn-to-stric
artifact: plan
---

# Plan

1. Move `#[default]` from `EnforcementMode::Warn` to `EnforcementMode::Strict` in
   `src/types.rs` and correct the variant doc comments. No logic change:
   `compute_exit_code` already implements the strict gate.
2. Pin the three integration tests whose fixtures carry real errors to
   `--enforcement warn`, with a comment recording why. They assert on reported output,
   not exit-code semantics.
3. Record the user-visible default change in `CHANGELOG.md`, including the opt-out.
