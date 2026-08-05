---
change: CHG-0084-give-the-change-module-canonical-ownership-of-its-cli-wiring
artifact: design
---

# Design

`specs/change/change.spec.md` adds `src/commands/change.rs` to its `files:` list.

The file is the CLI dispatch layer for the change lifecycle and belongs to the
same module as `src/change.rs`. A separate `cmd_change` spec would match the
`cmd_agents` precedent, but that pattern exists for commands whose module is
otherwise unrelated; here the wiring and the logic are one module.
