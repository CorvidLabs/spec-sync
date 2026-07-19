## ADDED

### REQUIREMENT REQ-cli-args-007

The shared CLI grammar SHALL expose the 5.0 ledger migration as an optional source-family
positional on the `migrate` command.

Acceptance Criteria

- `specsync migrate 5.0` selects the ledger backfill mode; bare `specsync migrate` keeps the
  v3→v4 default.
- An unknown source family fails through a deterministic Clap validation error before any
  mutation.
- `--dry-run` and `--no-backup` remain accepted in both modes.

## MODIFIED

### SPEC SECTION Invariants

1. All global flags use `#[arg(global = true)]` so they work regardless of subcommand position
2. `--json` is a shorthand alias for `--format json` — both set the same output format
3. `--enforcement` accepts three modes matching `types::EnforcementMode`: warn, enforce-new, strict
4. Default output format is `text` when neither `--json` nor `--format` is specified
5. The `Command` enum is optional — running `specsync` with no subcommand defaults to `Check`
6. Each `HooksAction::Install` / `Uninstall` variant carries identical boolean flags for symmetric install/uninstall
7. Each `AgentsAction::Install` / `Uninstall` variant carries identical boolean flags for symmetric install/uninstall, mirroring `HooksAction`
8. `Generate` exposes only deterministic uncovered/batch selection; provider and model flags are not accepted
9. `ChangeAction::Reopen` requires both `--actor` and `--reason`; neither can be omitted from the CLI grammar
10. `ChangeAction::CorrectOwner` requires exact path, canonical spec module, actor, and reason inputs
11. `Migrate` accepts an optional source-family positional; unknown families fail through deterministic validation before any mutation.
