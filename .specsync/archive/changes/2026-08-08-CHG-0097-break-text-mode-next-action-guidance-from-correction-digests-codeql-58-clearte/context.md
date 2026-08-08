---
change: CHG-0097-break-text-mode-next-action-guidance-from-correction-digests-codeql-58-clearte
artifact: context
---

# Context

## Problem

CodeQL alert #58 (`rust/cleartext-logging`, High) reports that text-mode
`change status` / `show` / `list` writes data from
`validate_trusted_correction_history` into a cleartext sink (`println!`) at
`src/commands/change.rs:545` on main.

The taint path on main was:

1. `text_mode_next_action` → `artifacts_complete_for_guidance`
2. → `validate_artifacts` → `effective_change_definition`
3. → `validate_trusted_correction_history` (correction digests / ledger)
4. → format string returned into `println!("  Next: …")`

A prior comment claimed guidance was "lightweight / no digests", but
`artifacts_complete_for_guidance` still called full `validate_artifacts`, which
loads the trusted correction history.

## Fix

- Split body checks into `validate_artifact_bodies(root, id, selected)`.
- `artifacts_complete_for_guidance` uses only the **persisted**
  `record.selected_artifacts` list (no ledger, no digests).
- `validate_artifacts` (used by approve) still re-checks against the
  effective correction-applied selection.
- Text printing helpers (`print_change_text_identity` /
  `print_change_text_answers`) keep human sinks free of digest-bearing loaders.
- JSON mode still emits `effective_definition` and `corrections` for machines.

## Constraints

- Do not weaken approve-time validation of effective selected artifacts.
- Do not change public API surface of `artifacts_complete_for_guidance` (already
  documented); only its digests-free behavior becomes true.
- Digests remain available via `--json`; humans get counts and a pointer to JSON.
