# Lesson bundle: change-check-and-change-audit-say-plainly-when-sdd-json-configures-verification-commands-they-did-not-run-so-a-pass-is

Material for folding this change's lessons into each affected module's lesson file.
Synthesise from what actually happened below; do not restate the change description.

## What this change was

- **Title**: Change check and change audit say plainly when sdd.json configures verification commands they did not run, so a pass is not read as the project's tests passing (#893)
- **Kind**: Feature
- **Specs**: cmd_change
- **Paths**: src/commands/change.rs, tests/integration/change.rs, .specsync/sdd.json, site/src/content/docs/configuration.md, docs/ADOPTING.md, specs/cmd_change/, CHANGELOG.md, hi/ship.md, INTENT.md, .chough/
- **Acceptance**: Text change check (plain and --commit) and change audit print one stderr note beginning `note: .specsync/sdd.json configures N verification command(s) that SpecSync did not run`, with singular `command` for N=1 and the words `to stop this note`.
- **Acceptance**: JSON check and audit output carries a top-level configured_commands_not_run list beside verification or report: verification commands first, component commands by module, then strict commands, each command once, and [] when none is configured.
- **Acceptance**: The disclosure never changes exit status and never spawns a configured command; change check --commit --json gains no field because that path has no JSON success output (#839).
- **Acceptance**: This repository empties only verification_commands while retaining all 13 component_verification_commands modules and all 3 strict_verification_commands, so its own note remains visible.
- **Acceptance**: The committed text, JSON, singular, empty-policy, no-spawn, and repository-policy tests fail before implementation and pass afterward, with the full integration suite updated for the additive stderr note where needed.

## Evidence

- Verification commit: `cbfb99d0c68914ac12bf99c4f6417b9a2df6e602`
- Base commit: `22a2dcaca598949dfb3a34907ea2fb0cfd18a820`
- Verified by: `specsync check --spec cmd_change`

## From the change's context.md

# Context

Issue #893: `change check` and `change audit` do not execute configured verification commands, but a passing result gives an adopter no visible reminder of that limit. SHIP-1.c.3 asks both commands to disclose what they did not run so a pass is not mistaken for the project's tests passing.

The committed integration probes failed first on b3ca5f97, recorded as OBS-product-test-693137d491cd153adf290ec2. They cover plain and committed checks, audit, JSON, an empty policy, singular wording, no process spawn, and this repository's policy.

Leif first approved emptying this repository's top-level `verification_commands`. His narrowed 3b decision, relayed by orc t18 on 2026-11-06, is authoritative: empty only that list; keep all 13 component modules and all 3 strict commands. Emptying the other lists to silence the note is rejected. The repository contract pin has been narrowed accordingly and still fails only on the four top-level commands.

## From the change's design.md

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

## From the change's testing.md

# Testing

Failing first on b3ca5f97 with implementation unchanged (OBS-product-test-693137d491cd153adf290ec2): plain check, committed check, and audit have no stderr note; JSON has no `configured_commands_not_run`; the repository pin finds four top-level commands. After Leif's narrowed 3b decision, the repository pin was rerun and fails only on those four top-level commands while its 13-component and 3-strict retention assertions pass.

After implementation, run all committed probes:

- `change_check_notes_the_configured_commands_it_did_not_run`: three lists in policy order, deduplication, exact stderr prefix, `to stop this note`, no stdout contamination, and no spawned reporter.
- `change_check_commit_notes_the_configured_commands_it_did_not_run`: the committed path prints the note.
- `change_audit_notes_the_configured_commands_it_did_not_run`: text note and JSON list.
- `an_empty_policy_gets_no_note_about_unrun_commands`: no text note and explicit `[]` in check and audit JSON, including empty verification output.
- `this_repository_configures_no_verification_command`: top-level list empty; 13 component modules and 3 strict commands retained.

Run the complete integration suite to locate additive-stderr incompatibilities, then `target/release/specsync check --strict --require-coverage 100 --force`, `fledge lanes run verify`, and `fledge trust verify`. Record affected Chough cases and the full-lane result separately from native spec↔code verification.

## Requirement evidence

| Requirement | Evidence |
| --- | --- |
| REQ-cmd-change-unrun-commands-001 | The five probes above, full integration suite, and no-spawn marker assertion |

## Where these lessons go

- `specs/cmd_change/cmd_change.3md`
