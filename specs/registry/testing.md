---
spec: registry.spec.md
---

## Automated Testing

| Test File | Type | What It Covers |
|-----------|------|----------------|
| `src/registry.rs` inline tests | Unit | Validate Registry behavior close to implementation, especially `RemoteRegistry`, `RemoteSpec`, `has_spec`, `bool`, `spec_path`, `Option<&str>`, `fetch_remote_registry`, `fetch_remote_spec` |
| `tests/integration.rs` | Integration | Exercise Registry through project workflows and spec validation fixtures |

## Manual Testing

- [ ] Run `fledge spec check --strict` after changing Registry contracts or source files.
- [ ] Run `fledge run test` and confirm Registry unit/integration coverage still passes.
- [ ] Review examples in `registry.spec.md` against observed behavior when touching src/registry.rs.

## Edge Cases & Boundary Conditions

| Scenario | Expected Behavior |
|----------|-------------------|
| HTTP request fails | Error: "HTTP request failed: {details}" |
| Repo has no registry file | Error: "HTTP 404 — {repo} may not have a specsync-registry.toml" |
| Malformed TOML (no name) | `parse_registry` returns `None` |
| Local registry file unreadable | `load_registry` returns `None` |
