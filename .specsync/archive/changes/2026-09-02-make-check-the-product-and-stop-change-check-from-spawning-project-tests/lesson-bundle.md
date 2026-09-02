# Lesson bundle — make-check-the-product-and-stop-change-check-from-spawning-project-tests

Material for folding this change's lessons into the affected specs' `context.md`.
Synthesise from what actually happened below; do not restate the change description.

## What this change was

- **Title**: Make check the product and stop change check from spawning project tests
- **Kind**: Feature
- **Specs**: change, cmd_change, cmd_check, cmd_init, agents
- **Paths**: src/change.rs, src/change_tests.rs, tests/integration/change.rs, tests/integration/comment.rs, site/src/content/docs/cli.md, site/src/content/docs/configuration.md, site/src/content/docs/quickstart.md, site/src/content/docs/workflow.md, docs/ADOPTING.md, MIGRATION.md, CHANGELOG.md, specs/agents/agents.spec.md, specs/agents/requirements.md, specs/change/change.spec.md, specs/change/requirements.md, specs/change/testing.md, specs/cmd_change/cmd_change.spec.md, specs/cmd_check/cmd_check.spec.md, specs/cmd_init/cmd_init.spec.md, specs/cmd_init/requirements.md
- **Acceptance**: Fresh init writes SDD off and does not start a first-change interview.
- **Acceptance**: specsync check does not call audit_project or print an active-change count.
- **Acceptance**: change check compares specs to code in-process and does not spawn sdd.json verification_commands.
- **Acceptance**: A configured verification_commands sentinel is not executed.
- **Acceptance**: A phantom export still fails change check.
- **Acceptance**: change audit no longer re-runs project test commands in CI.

## Evidence

- Verification commit: `7ef1b442227410f9318d1b9b2dd8d151d594ca49`
- Base commit: `359eeee2981f72ce915a65ccc36ade84127d93a9`
- Verified by: `specsync check --spec agents --spec change --spec cmd_change --spec cmd_check --spec cmd_init`

## From the change's context.md

# Context

`specsync check` is the product: look at specs, look at code, report drift. The SDD change
workflow is opt-in (`specsync change adopt`). `change check` stole the word "check" and then
spawned `sdd.json` `verification_commands` (`cargo test` on this repo), so a spec-code tool
spent 15–20 minutes running the project's tests.

This change is already implemented on the branch. The workspace exists so this repo's still-on
SDD path coverage can see the dirty files.

Constraints: quiet 6.0 candidate. Do not merge to main. Do not tag. Do not cut rc.12.
This repo's `.specsync/sdd.json` stays `enabled: true` so `change` still works here.

## From the change's design.md

# Design

No UI. CLI only. `change check` records one `CommandEvidence` row named `specsync check` or
`specsync check --strict`. Status projections list that same command instead of the policy
`verification_commands` list.

## From the change's testing.md

# Testing

## Discriminators

- `change_check_does_not_execute_configured_project_commands` — python sentinel file must not appear.
- `change_check_does_not_spawn_a_configured_verification_child` — reporter script never writes `group.txt`.
- `change_check_does_not_wait_on_a_held_cargo_build_lock` — held Cargo locks are not named.
- `check_does_not_walk_sdd_when_policy_is_on` — `specsync check` stays silent on a corrupt workspace.

## Control

- `change_check_fails_when_specs_and_code_drift` — phantom export `does_not_exist` fails.
- `failed_spec_sync_is_retryable_with_append_only_history` — failed then passing spec sync keeps attempts.

## Requirement evidence

| Requirement | Evidence |
|-------------|----------|
| REQ-change-023 | `src/change_tests.rs` `failed_spec_sync_is_retryable_with_append_only_history` |
| REQ-change-049 | `src/change_tests.rs` `change_check_does_not_execute_configured_project_commands`, `change_check_fails_when_specs_and_code_drift` |
| REQ-change-050 | `src/commands/init.rs` `fresh_init_leaves_sdd_off_so_the_first_check_is_just_drift` |
| REQ-change-058 | `src/change.rs` `evaluate_spec_code_sync`; `check_project_with_command_output` no longer loops `run_configured_command` |
| REQ-change-091 | `tests/integration/change.rs` `change_check_does_not_wait_on_a_held_cargo_build_lock`, `change_check_does_not_spawn_a_configured_verification_child` |
| REQ-cmd-check-004 | `tests/integration/commands.rs` `check_does_not_walk_sdd_when_policy_is_on` |
| REQ-cmd-init-005 | `src/commands/init.rs` `fresh_init_leaves_sdd_off_so_the_first_check_is_just_drift` |
| REQ-agents-check-audit-commands-001 | `src/agents.rs` `install_claude_creates_skill_and_command`; generated check skill says spec↔code sync |

## Where these lessons go

- `specs/change/context.md`
- `specs/cmd_change/context.md`
- `specs/cmd_check/context.md`
- `specs/cmd_init/context.md`
- `specs/agents/context.md`
