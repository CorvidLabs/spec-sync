---
spec: archive.spec.md
---

## Automated Testing

| Test File | Type | What It Covers |
|-----------|------|----------------|
| `src/archive.rs` inline tests | Unit | Validate Archive behavior close to implementation, especially `archive_tasks`, `Vec<ArchiveResult>`, `count_completed_tasks`, `usize`, `ArchiveResult` |
| `tests/integration.rs` | Integration | Exercise Archive through project workflows and spec validation fixtures |

## Manual Testing

- [ ] Run `fledge spec check --strict` after changing Archive contracts or source files.
- [ ] Run `fledge run test` and confirm Archive unit/integration coverage still passes.
- [ ] Review examples in `archive.spec.md` against observed behavior when touching src/archive.rs.

## Edge Cases & Boundary Conditions

| Scenario | Expected Behavior |
|----------|-------------------|
| tasks.md file unreadable | Prints error in red, continues processing other files |
| tasks.md file unwritable | Prints error in red, continues processing other files |
