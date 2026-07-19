---
change: CHG-0059-tolerate-inert-5-0-1-registry-toml-stubs-so-module-resolution-falls-back-to-defa
artifact: testing
---

# Testing

- `REQ-registry-002`: unit coverage detects inert stubs (empty, `[registry]` only, `[specs]` only, and the 5.0.1 `version=1`/`[modules]` placeholder), loads named registries, and fails closed on non-inert unparsable files through `load_local_registry`.
- `REQ-change-041`: unit coverage proves `canonical_module_paths` succeeds against an inert stub via conventional `specs/<module>/` paths and still emits the exact parse diagnostic for non-inert unparsable registries.
