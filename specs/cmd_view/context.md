---
spec: cmd_view.spec.md
---

## Key Decisions

- Thin command wrapper: load config, find spec files, loop and delegate each to `view::view_spec`, print with a `---` separator. All section-filtering logic lives in the `view` library module.
- Per-file resilience: a render error for one spec is printed to stderr and skipped rather than aborting the whole run.
- Role validation is delegated: an unknown role surfaces as an error string from `view::view_spec` (valid roles: dev, qa, product, agent).
- `--spec` filtering matches on module name derived from the file stem with a trailing `.spec` stripped.

## Files to Read First

- `src/commands/view.rs` — the command wrapper
- `src/view.rs` — `view_spec`, `sections_for_role`, `valid_roles` (the actual role-to-section mapping)
- `src/validator.rs` — `find_spec_files` used for discovery

## Current Status

Fully implemented and stable. The command itself has no unit tests; the role/section logic it relies on is unit-tested in `src/view.rs` (e.g. `test_sections_for_role`).

## Notes

- Part of the command layer — it orchestrates the `view`, `config`, and `validator` modules rather than holding domain logic.
