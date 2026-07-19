---
change: CHG-0059-tolerate-inert-5-0-1-registry-toml-stubs-so-module-resolution-falls-back-to-defa
artifact: tasks
---

# Tasks

- [x] Add `is_inert_legacy_registry_stub` and Result-based `load_local_registry`
- [x] Route `canonical_module_paths` through `load_local_registry`
- [x] Add unit coverage for inert tolerate and non-inert fail-closed paths
- [x] Map REQ-registry-002 and REQ-change-041; update module companions
- [x] Run pre-acceptance formatting, lint, tests, and trust gate
