---
spec: cmd_agents.spec.md
---

## Tasks

- [ ] Add integration tests covering `agents install`/`uninstall`/`status` CLI behavior (currently no fixtures exist for this command, matching `cmd_hooks`'s same known gap).

## Done

- [x] `cmd_agents` dispatcher implemented for `Install`, `Uninstall`, and `Status`.
- [x] `collect_agent_targets` maps the four boolean flags (claude, cursor, codex, gemini) to `agents::AgentTool` variants.
- [x] Empty-target-vec convention ("install/uninstall all") wired through to the `agents` module.

## Gaps

No integration or inline unit tests target `src/commands/agents.rs`. The behavior is exercised only indirectly via the `agents` library module's own tests.

## Review Sign-offs

- **Product**: pending
- **QA**: pending
- **Design**: n/a
- **Dev**: pending
