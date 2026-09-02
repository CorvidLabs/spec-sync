---
change: make-check-the-product-and-stop-change-check-from-spawning-project-tests
artifact: research
---

# Research

`change check` used `verification_commands_for_change` then `run_configured_command`. On this
repo that list includes `cargo test`, which is why a spec-code verifier took 15–20 minutes.
`specsync check` already did the spec↔code pass in-process via `validate_spec`.
`evaluate_spec_code_sync` is that pass, with ignore-rule suppression, without spawning a child.

Phantom exports are errors (`Spec documents X but no matching export`). Undocumented exports
are warnings, so default (non-strict) verify still passes a fixture whose Public API is `None`.
The control uses a phantom, not an undocumented export.
