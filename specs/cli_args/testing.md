---
spec: cli_args.spec.md
---

## Regression Matrix

| Case | Required Result |
|------|-----------------|
| No subcommand | Defaults to check dispatch |
| Global flags before/after command | Equivalent parse result |
| `generate --provider` | Clap rejects unknown argument |
| `generate --model` | Clap rejects unknown argument |
| `generate --batch` | Collects requested modules |
| Agents/MCP/Change | Commands continue parsing |
| `change reopen <id> --actor <human> --reason <text>` | Parses the audited transition inputs |
| `change reopen` missing actor/reason | Clap rejects the incomplete command |
