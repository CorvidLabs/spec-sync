---
change: CHG-0059-tolerate-inert-5-0-1-registry-toml-stubs-so-module-resolution-falls-back-to-defa
artifact: design
---

# Design

Classify a local registry file as inert when scan finds no non-empty registry `name` and no `[specs]`
mappings. Empty `[modules]` tables (5.0.1 placeholders) do not count as mappings. `load_local_registry`
returns `Ok(None)` for missing/inert, `Ok(Some)` for named parse success, and `Err` for non-inert
unparsable content. `load_registry` remains best-effort via `.ok().flatten()`.
`canonical_module_paths` switches from existence+`load_registry` to `load_local_registry` so inert
stubs take the same path as a missing file.
