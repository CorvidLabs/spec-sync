---
spec: hash_cache.spec.md
---

## Automated Testing

| Test File | Type | What It Covers |
|-----------|------|----------------|
| `src/hash_cache.rs` inline tests | Unit | Validate Hash Cache behavior close to implementation, especially `HashCache`, `ChangeKind`, `ChangeClassification`, `load`, `Self`, `save`, `io::Result<()>`, `hash_file` |
| `tests/integration.rs` | Integration | Exercise Hash Cache through project workflows and spec validation fixtures |

## Manual Testing

- [ ] Run `fledge spec check --strict` after changing Hash Cache contracts or source files.
- [ ] Run `fledge run test` and confirm Hash Cache unit/integration coverage still passes.
- [ ] Review examples in `hash_cache.spec.md` against observed behavior when touching src/hash_cache.rs.

## Edge Cases & Boundary Conditions

| Scenario | Expected Behavior |
|----------|-------------------|
| Cache file missing | Returns empty cache (all files treated as changed) |
| Cache file has invalid JSON | Returns empty cache silently |
| File unreadable during hashing | `hash_file` returns `None`; file treated as changed |
| Cannot create `.specsync/` directory | `save` returns `io::Error` |
