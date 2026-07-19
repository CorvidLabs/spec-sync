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
| `change correct <id> <field> <value> --actor <human> --reason <text>` | Parses only the supported audited correction grammar (`REQ-cli-args-004`) |
| `change correct` with an unsupported field/value or missing actor/reason | Clap rejects the command before domain mutation |
| `change correct-owner <id> --path <path> --spec <module> --actor <human> --reason <text>` | Parses the complete audited exact-owner correction grammar (`REQ-cli-args-005`) |
| `change correct-owner` missing path/spec/actor/reason | Clap rejects the command before domain mutation |
| `change correct-owner` with repeated `--path`, `--manifest`, or `--all-missing` | Parses batch selection grammar (`REQ-cli-args-006`) |
| `migrate 5.0 [--dry-run]` | Parses the ledger backfill mode (`REQ-cli-args-007`) |
| `migrate 9.9` | Clap rejects the unknown source family before domain mutation |
