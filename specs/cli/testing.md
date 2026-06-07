---
spec: cli.spec.md
---

## Automated Testing

| Test File | Type | What It Covers |
|-----------|------|----------------|
| `src/main.rs` inline tests | Unit | Validate Cli behavior close to implementation, especially `coverage`, `score`, `init`, `view`, `diff`, `compact`, `archive-tasks`, `deps` |
| `tests/integration.rs` | Integration | Exercise Cli through project workflows and spec validation fixtures |

## Manual Testing

- [ ] Run `fledge spec check --strict` after changing Cli contracts or source files.
- [ ] Run `fledge run test` and confirm Cli unit/integration coverage still passes.
- [ ] Smoke-test `cargo run -- cli --help` or the nearest CLI path that routes through this module.

## Edge Cases & Boundary Conditions

| Scenario | Expected Behavior |
|----------|-------------------|
| Cannot determine cwd | Panics with "Cannot determine cwd" |
| AI provider not found (with `--provider`) | Prints error to stderr and exits 1 |
| Failed to write `specsync.json` | Panics with "Failed to write specsync.json" |
| Failed to create spec directory | Prints error to stderr and exits 1 |
