---
change: CHG-0128-every-command-that-derives-a-module-s-api-must-honour-the-configured-export-leve
artifact: design
---

# Design

Thread `&SpecSyncConfig` to every site that derives a module's API, and take
both `export_level` and `parse_mode` from it. The wrapper's convenience was
exactly its danger: it let a caller obtain "the exports" without stating which
surface it meant, and five callers took the default without noticing.

Deleting the wrapper outright would have been the stronger guarantee — a
removed function cannot be called by mistake. That was tried and reverted: this
repository's own `specs/exports/exports.spec.md` documents both wrapper names in
its Public API, so removing them failed spec-sync's own drift check. Rather than
hand-edit living specs to make an implementation convenient, both are retained
as `#[allow(dead_code)]` with the #474 warning in their doc comments, matching
the pattern already used by `get_exported_symbols_with_level` in the same file.

That is a weaker guard and worth naming: the protection is a doc comment and two
tests, not the type system. A future caller can still reach for the short name.
The honest fix is to delete the wrappers and retire them from the spec in one
change; that is a deliberate contract change and does not belong inside a bug
fix.

Signature changes are confined to the bin crate — `generate_spec`,
`generate_spec_from_custom_template`, `collect_exports_for_files`,
`render_spec_template`, `generate_module_spec` each take `&SpecSyncConfig`.
Every caller already had the loaded config in scope. No exported symbol names
change, so the repo's own `check` is unaffected.
