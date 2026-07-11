---
spec: cmd_hooks.spec.md
---

## Tasks

- [x] Add integration coverage for `hooks install` and `hooks uninstall` — Evidence: `hooks_install_claude_code_hook_preserves_user_settings` and `hooks_uninstall_preserves_user_content_after_block`.

## Post-5.0 Test Debt

- [ ] Add integration coverage for `hooks status` CLI behavior.

## Done

- [x] `cmd_hooks` dispatcher implemented for `Install`, `Uninstall`, and `Status`.
- [x] `collect_hook_targets` maps the six boolean flags (claude, cursor, copilot, agents, precommit, claude_code_hook) to `hooks::HookTarget` variants.
- [x] Empty-target-vec convention ("install/uninstall all") wired through to the `hooks` module.

## Gaps

- No integration or inline unit tests target `src/commands/hooks.rs`. The behavior is exercised only indirectly via the `hooks` library module's own tests.

## Review Status

Per-module role sign-offs were not collected. Release approval is governed by digest-bound change approvals and required CI; this note is informational and is not a release gate.
