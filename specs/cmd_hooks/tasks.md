---
spec: cmd_hooks.spec.md
---

## Tasks

- [ ] Add integration tests covering `hooks install`/`uninstall`/`status` CLI behavior (currently no fixtures exist for this command).

## Done

- [x] `cmd_hooks` dispatcher implemented for `Install`, `Uninstall`, and `Status`.
- [x] `collect_hook_targets` maps the six boolean flags (claude, cursor, copilot, agents, precommit, claude_code_hook) to `hooks::HookTarget` variants.
- [x] Empty-target-vec convention ("install/uninstall all") wired through to the `hooks` module.

## Gaps

- No integration or inline unit tests target `src/commands/hooks.rs`. The behavior is exercised only indirectly via the `hooks` library module's own tests.

## Review Sign-offs

- **Product**: pending
- **QA**: pending
- **Design**: n/a
- **Dev**: pending
