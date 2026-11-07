---
spec: cli_args.spec.md
---

## Tasks

(none open for 5.0)

## Done

- [x] Add `mcp --allow-write` with a read-only default and configured-root help text
- [x] Correct initialization and full-scaffold help for the 5.0 layout
- [x] Preserve global flags and complete command grammar
- [x] Preserve verified Change, Agents, MCP, and lifecycle namespaces
- [x] Remove provider/model flags and add rejection regressions
- [x] Add required actor/reason grammar for audited accepted-change reopen
- [x] Add closed field/value grammar and required audit inputs for accepted metadata correction
- [x] Add exact path/spec grammar and required audit inputs for acceptance-owner correction
- [x] Add batch selection grammar for correct-owner (repeated path/spec, manifest, all-missing)
- [x] Add optional migrate source-family positional for the 5.0 ledger backfill
- [x] Add an explicit pass/block verdict to the plain scoped-review command

- [x] Clarify scoped-review claims versus separately enforced authentication without changing runtime review rules.
- [x] Parse the relayed-decision flags on `change approve` and `change review` and say the actor and reviewer are recorded claims (REQ-cli-args-relayed-decision-001, #633).
- [x] Parse `change repair-relays <id>`, taking only the change id (#904, REQ-cli-args-relay-repair-001).
