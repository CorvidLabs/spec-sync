## ADDED

### REQUIREMENT REQ-cmd-hooks-001

The hooks command SHALL map CLI actions and target flags to the hook subsystem without duplicating file-management policy.

Acceptance Criteria
- `cmd_hooks` dispatches `Install`, `Uninstall`, and `Status` actions to `hooks::cmd_install`, `hooks::cmd_uninstall`, and `hooks::cmd_status` respectively.
- Boolean flags `claude`, `cursor`, `copilot`, `agents`, `precommit`, `claude_code_hook` map one-to-one to `hooks::HookTarget` variants in `collect_hook_targets`.
- When no target flags are set, the collected target vec is empty, which the `hooks` module interprets as "all targets".
- The same flag-to-target mapping is used for both install and uninstall.

## MODIFIED

### SPEC SECTION Dependencies

**Consumes**

| Module | What is used |
|--------|-------------|
| hooks | `cmd_install`, `cmd_uninstall`, `cmd_status` |
| cli_args | `HooksAction` enum |

**Consumed By**

| Module | What is used |
|--------|-------------|
| cli (main.rs) | Entry point for `specsync hooks` |

**Frontmatter Synchronization**

Implementation SHALL add these canonical dependency specs to `depends_on`: `specs/cli/cli.spec.md`. This YAML frontmatter update is an explicit implementation edit because semantic section deltas do not apply frontmatter.
