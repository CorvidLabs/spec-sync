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
