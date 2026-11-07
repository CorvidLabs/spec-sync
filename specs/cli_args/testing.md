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
| `mcp` | Parses with `allow_write: false` and lists only read tools |
| `mcp --allow-write` | Parses with `allow_write: true`; help names the configured project root |
| `change reopen <id> --actor <human> --reason <text>` | Parses the audited transition inputs |
| `change reopen` missing actor/reason | Clap rejects the incomplete command |
| `change correct <id> <field> <value> --actor <human> --reason <text>` | Parses only the supported audited correction grammar (`REQ-cli-args-004`) |
| `change correct` with an unsupported field/value or missing actor/reason | Clap rejects the command before domain mutation |
| `change correct-owner <id> --path <path> --spec <module> --actor <human> --reason <text>` | Parses the complete audited exact-owner correction grammar (`REQ-cli-args-005`) |
| `change correct-owner` missing path/spec/actor/reason | Clap rejects the command before domain mutation |
| `change correct-owner` with repeated `--path`, `--manifest`, or `--all-missing` | Parses batch selection grammar (`REQ-cli-args-006`) |
| `migrate 5.0 [--dry-run]` | Parses the ledger backfill mode (`REQ-cli-args-007`) |
| `migrate 9.9` | Clap rejects the unknown source family before domain mutation |


## Historical reconciliation evidence

REQ-cli-args-020: change_historical_reconciliation_requires_explicit_actor_and_reason

REQ-cli-args-021: change_send_back_requires_and_collects_audit_inputs

REQ-cli-args-relayed-decision-001: change_approve_and_review_take_an_optional_relayed_decision; change_check_review_and_finalize_are_plain_commands still parses review without the flags.

REQ-cli-args-relay-repair-001 (#904): change_repair_relays_takes_only_the_change_id (src/cli.rs): the id parses, a missing id or any other argument fails, `change --help` lists `repair-relays`, and `change status` still parses.
