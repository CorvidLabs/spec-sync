## ADDED

### REQUIREMENT REQ-cli-args-005

The shared CLI grammar SHALL expose one explicit audited command for correcting an exact canonical
owner used by reopened acceptance evidence.

Acceptance Criteria

- `change correct-owner` requires a change ID, exact portable path, canonical spec module, human
  actor, and non-empty reason.
- The canonical module is provided through `--spec` and remains distinct from semantic rescoping.
- Missing path, module, actor, or reason inputs fail through deterministic Clap errors before any
  domain mutation.

## MODIFIED

### SPEC SECTION Public API

**Exported Structs**

| Type | Description |
|------|-------------|
| `Cli` | Top-level Clap parser struct with global flags (`--strict`, `--root`, `--format`, `--json`, `--enforcement`, `--require-coverage`) and a subcommand field |

**Exported Enums**

| Type | Description |
|------|-------------|
| `Command` | Root subcommand enum including canonical validation, module lifecycle, and verified SDD Change operations |
| `HooksAction` | Sub-subcommand for `Hooks`: Install, Uninstall, Status — each with boolean flags for target selection (claude, cursor, copilot, agents, precommit, claude_code_hook) |
| `AgentsAction` | Sub-subcommand for `Agents`: Install, Uninstall, Status — each with boolean flags for target selection (claude, cursor, codex, gemini) |
| `LifecycleAction` | Sub-subcommand for `Lifecycle`: Promote, Demote, Set, Status, History, Guard, AutoPromote, Enforce — manages spec lifecycle transitions |
| `ChangeAction` | Sub-subcommand for `Change`: New, Answer, Depend, Supersede, List, Show, Status, Approve, Start, Verify, Reopen, Correct, CorrectOwner, Accept, Archive, Check, Adopt |

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

### SPEC SECTION Error Cases

| Condition | Behavior |
|-----------|----------|
| Unknown subcommand | Clap prints error with usage help and exits non-zero |
| Missing required argument (e.g. `new` without name) | Clap prints error listing required args |
| Invalid `--enforcement` value | Clap prints accepted values: warn, enforce-new, strict |
| Invalid `--format` value | Clap prints accepted values: text, json, markdown, github, table, csv |
| `change reopen` without `--actor` or `--reason` | Clap names the missing required argument and exits non-zero |
| `change correct-owner` without path, spec, actor, or reason | Clap names the missing required argument and exits non-zero |
