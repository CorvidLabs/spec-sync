---
spec: manifest.spec.md
---

## Automated Testing

| Test File | Type | What It Covers |
|-----------|------|----------------|
| `src/manifest.rs` inline tests | Unit | Validate Manifest behavior close to implementation, especially `ManifestModule`, `ManifestDiscovery`, `discover_from_manifests` |
| `tests/integration.rs` | Integration | Exercise Manifest through project workflows and spec validation fixtures |

## Manual Testing

- [ ] Run `fledge spec check --strict` after changing Manifest contracts or source files.
- [ ] Run `fledge run test` and confirm Manifest unit/integration coverage still passes.
- [ ] Review examples in `manifest.spec.md` against observed behavior when touching src/manifest.rs.

## Edge Cases & Boundary Conditions

| Scenario | Expected Behavior |
|----------|-------------------|
| Manifest file missing | Parser returns `None`, skipped silently |
| Manifest file unreadable | Parser returns `None` (fs::read_to_string fails gracefully) |
| Malformed manifest content | Best-effort extraction; missing fields result in defaults or skipped entries |
| Workspace member directory doesn't exist | Skipped (Cargo.toml existence check) |
