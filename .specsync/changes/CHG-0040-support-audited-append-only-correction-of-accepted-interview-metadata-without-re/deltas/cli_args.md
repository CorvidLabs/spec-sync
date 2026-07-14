## ADDED

### REQUIREMENT REQ-cli-args-004

The shared CLI grammar SHALL expose a complete explicit command for supported accepted interview
metadata correction.

Acceptance Criteria

- `change correct` requires a change ID, supported field, `yes` or `no` value, human actor, and
  non-empty reason input.
- Help distinguishes accepted metadata correction from delivery-only `change reopen`.
- Missing audit arguments and invalid field/value choices fail through deterministic Clap errors.

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
| `ChangeAction` | Sub-subcommand for `Change`: New, Answer, Depend, List, Show, Status, Approve, Start, Verify, Reopen, Correct, Accept, Archive, Check, Adopt |
