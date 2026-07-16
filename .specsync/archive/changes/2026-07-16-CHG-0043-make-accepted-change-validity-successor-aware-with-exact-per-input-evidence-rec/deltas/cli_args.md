## MODIFIED

### REQUIREMENT REQ-cli-args-001

The system SHALL declare the complete verified SDD change command grammar in the shared Clap parser.

Acceptance Criteria
- `Command` includes the `Change` namespace.
- `ChangeAction` declares every lifecycle, inspection, checking, adoption, and semantic-succession operation.
- `ChangeAction::Supersede` requires change ID, predecessor ID, path, module, and predecessor entry digest.
- `ChangeAction::Reopen` requires a change ID, explicit human actor, and non-empty reason input.
- `ChangeAction::Approve` exposes an explicit `--portable-5-0-1` switch for the atomic marked dual-engine definition event.


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
| `ChangeAction` | Sub-subcommand for `Change`: New, Answer, Depend, Supersede, List, Show, Status, Approve, Start, Verify, Reopen, Accept, Archive, Check, Adopt |
