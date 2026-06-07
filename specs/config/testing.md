---
spec: config.spec.md
---

## Automated Testing

| Test File | Type | What It Covers |
|-----------|------|----------------|
| `src/config.rs` inline tests | Unit | Validate Config behavior close to implementation, especially `load_config`, `SpecSyncConfig`, `load_config_from_path`, `detect_source_dirs`, `Vec<String>`, `default_schema_pattern`, `discover_manifest_modules`, `ManifestDiscovery` |
| `tests/integration.rs` | Integration | Exercise Config through project workflows and spec validation fixtures |

## Manual Testing

- [ ] Run `fledge spec check --strict` after changing Config contracts or source files.
- [ ] Run `fledge run test` and confirm Config unit/integration coverage still passes.
- [ ] Review examples in `config.spec.md` against observed behavior when touching src/config.rs.

## Edge Cases & Boundary Conditions

| Scenario | Expected Behavior |
|----------|-------------------|
| Config file unreadable | Falls back to `SpecSyncConfig::default()` |
| Malformed JSON config | Prints warning to stderr, falls back to defaults |
| Empty project root | Returns `["src"]` as source dirs |
