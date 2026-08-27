---
spec: cmd_wizard.spec.md
---

## Key Decisions

- Interactive-first: all input is gathered through `dialoguer` (`Input`, `Select`, `Confirm`). Interrupting any prompt maps to a clean `process::exit(0)`.
- Safety before writing: nothing touches disk until the user confirms the preview; an existing spec aborts the wizard early.
- Module-type presets are inlined as `(extra_invariants, extra_api_hint)` tuples keyed on the selected template index, so each type seeds appropriate invariants and an API table.
- The spec body is built as one `format!` string here rather than via the generator, but companion files are still produced through `generator::generate_companion_files_for_spec` (so design.md respects `companions.design`).

## Files to Read First

- `src/commands/wizard.rs` — the full interactive flow and spec-body template
- `src/generator.rs` — `generate_companion_files_for_spec` for the companion set
- `src/config.rs` — `source_dirs`, `source_extensions`, `companions.design`
- `src/exports/mod.rs` — `has_configured_extension` used during source auto-detection

## Current Status

Fully implemented and stable. No automated tests cover the interactive flow (it requires a TTY); the spec-body shape mirrors the `scaffold`/`generate` templates.

## Notes

- Part of the command layer — orchestrates `config`, `generator`, and `exports`; one of two commands that depend on interactive prompts (`init` is the other; both import `dialoguer`).
