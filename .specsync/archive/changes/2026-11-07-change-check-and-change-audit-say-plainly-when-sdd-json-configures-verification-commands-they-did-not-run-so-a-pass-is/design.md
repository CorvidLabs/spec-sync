---
change: change-check-and-change-audit-say-plainly-when-sdd-json-configures-verification-commands-they-did-not-run-so-a-pass-is
artifact: design
---

# Design

## One presentation helper

In `src/commands/change.rs`, a private helper reads `.specsync/sdd.json` and returns the configured commands in this order: `verification_commands`, each `component_verification_commands` module in deterministic module order, then `strict_verification_commands`. A missing policy means an empty list; malformed policy keeps the command's existing error. Stable deduplication keeps the first occurrence of each command. No command is executed.

The text formatter emits exactly one stderr line when the list is non-empty:

`note: .specsync/sdd.json configures N verification command(s) that SpecSync did not run; ... to stop this note ...`

`command` is singular only for N=1. The remainder names the lists to empty. Plain `change check`, its `--commit` path, and `change audit` all call the same formatter; the committed path is explicitly included because it prints its own summary. Text stdout is unchanged.

## Structured output

Successful JSON `change check` adds top-level `configured_commands_not_run` beside `verification`, including the `verification: null` case. JSON `change audit` adds the same key beside `report`. The key is always present, including `[]`. The committed JSON path has no success document today (#839), so it gains no synthetic output. JSON stdout remains one document and the text-only note is not emitted there.

## Repository policy and documentation

Only this repository's top-level `verification_commands` becomes `[]`. The 13 component modules and 3 strict commands stay, so SpecSync's own text checks continue to disclose them. The configuration and adoption guides explain the note and JSON field, and the changelog records the additive disclosure.

## Out of scope

No verification policy, command routing, exit-status, evidence-schema, or lifecycle-state change; no new domain export; no execution or scheduling of configured commands; no generic JSON work for `change check --commit` (#839); no removal of component or strict commands.
