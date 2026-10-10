---
spec: cmd_rules.spec.md
---

# cmd_rules — Context

## Design Decisions

- **Standing principles**: when `.specsync/sdd.json` names `principles_file`, the command prints that path and says a lesson becomes a standing rule only when a person writes it there. No file configured prints "none configured". The command still does not write anything.
- **Read-only command**: `rules` is purely informational — it reads config and displays, never modifies state.
- **Built-in rules always shown**: Even when all are "off", they're listed so users know what's available to configure.
- **Color-coded severity**: Matches the color scheme used in `specsync check` output for consistency.

## Related

- Custom rules are defined in `specsync.json` under the `customRules` key.
- Custom rule validation logic lives in `src/validator.rs`, not in this command module.
