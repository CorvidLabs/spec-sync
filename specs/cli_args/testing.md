---
spec: cli_args.spec.md
---

## Automated Testing

| Test File | Type | What It Covers |
|-----------|------|----------------|
| `src/cli.rs` inline tests | Unit | Validate Cli Args behavior close to implementation, especially `Cli`, `Command`, `HooksAction`, `LifecycleAction` |
| `tests/integration.rs` | Integration | Exercise Cli Args through project workflows and spec validation fixtures |

## Manual Testing

- [ ] Run `fledge spec check --strict` after changing Cli Args contracts or source files.
- [ ] Run `fledge run test` and confirm Cli Args unit/integration coverage still passes.
- [ ] Review examples in `cli_args.spec.md` against observed behavior when touching src/cli.rs.

## Edge Cases & Boundary Conditions

| Scenario | Expected Behavior |
|----------|-------------------|
| Unknown subcommand | Clap prints error with usage help and exits non-zero |
| Missing required argument (e.g., `new` without name) | Clap prints error listing required args |
| Invalid `--enforcement` value | Clap prints accepted values: warn, enforce-new, strict |
| Invalid `--format` value | Clap prints accepted values: text, json, markdown |
