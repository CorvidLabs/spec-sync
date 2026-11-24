# Lesson bundle: in-a-shallow-clone-name-git-fetch-unshallow-origin-in-every-lifecycle-history-refusal-and-refuse-change-new-before

Material for folding this change's lessons into each affected module's lesson file.
Synthesise from what actually happened below; do not restate the change description.

## What this change was

- **Title**: In a shallow clone name git fetch --unshallow origin in every lifecycle-history refusal and refuse change new before writing
- **Kind**: BugFix
- **Specs**: change, cmd_change
- **Paths**: src/change/validate_loaded_change.rs, src/change/sdd_version.rs, src/commands/change.rs, src/change_tests.rs, src/change_tests/git_environments.rs, tests/integration.rs, tests/integration/, specs/change/, specs/cmd_change/, site/src/content/docs/, CHANGELOG.md
- **Acceptance**: (1) In a depth-1 and a depth-50 clone of an adopted project, change new, change status, change check and change audit refuse, saying the clone is shallow and naming git fetch --unshallow origin; after that fetch they behave as in a full clone. (2) A refused change new leaves no .specsync/changes/<id>/. (3) In a full clone a baseline whose cutoff is not an ancestor keeps today's message, and a corrupt correction ledger keeps its own. (4) The tester's two shapes (a depth-1 clone with no origin ref; a shallow clone with origin/main but not the merge-base) name the fetch.

## Evidence

- Verification commit: `6217fc589342ac4753c8396476e720f9016756f2`
- Base commit: `ed258f58d734be7c4c0ac092ddbc934f492630d3`
- Verified by: `specsync check --spec change --spec cmd_change`

## From the change's context.md

# Context

Issue #1072. In a shallow clone of an adopted project, `change new` exits 0 and writes a change that no other command can use: `change status`, `check` and `audit` refuse with `workflow-v2 baseline cutoff must be an ancestor of its introduction's first parent`, and a history failure under the correction ledger reads as `correction ledger integrity is invalid`. After `git fetch --unshallow origin` the same change reads normally (measured on spec-sync, section 3 of the definition).

Definition: ss-lead/gates/git-environments-909-1072-1077-definition.md, section 3 and REQ-change-173. Leif's choices 3 and 4 in the tester's dialog of 2026-11-20 at about 00:45Z: `change new` refuses before writing, and every such refusal in a shallow clone names the fetch.

## From the change's design.md

# Design

- **One shallow test**, `git rev-parse --is-shallow-repository`, asked only when a lifecycle-history check has already refused, so a full clone spawns nothing new.
- **The baseline history check** (`validate_workflow_v2_baseline_history`, through `validate_workflow_v2_baseline`): in a shallow clone, any refusal becomes the shallow message, naming the cutoff and `git fetch --unshallow origin`.
- **The correction-ledger text** (`load_change_for_definition_mutation`, `ensure_text_correction_ledger_valid`): in a shallow clone, the same shallow message.
- **`change new`** asks the same history check before it allocates a workspace, only in a shallow clone of an adopted project, and refuses with the `change new` form of the message, writing nothing (REQ-change-108's order).
- **A full clone** keeps every message as it is.

## From the change's testing.md

# Testing

## Requirement evidence

| Requirement | Evidence |
|---|---|
| REQ-change-173 | `tests/integration/git_environments.rs`: in depth-1 and depth-50 clones of an adopted project, `change new` refuses naming the fetch and leaves no workspace, `change status`, `check` and `audit` name the fetch, and after `git fetch --unshallow origin` they behave as in a full clone; a full clone keeps today's messages; the tester's two shapes name the fetch |
| REQ-cmd-change-044 | `tests/integration/git_environments.rs`: in a shallow clone deep enough for the baseline but not for a change's base commit, text `change status <id>` names the shallow clone and `git fetch --unshallow origin`; in a full clone a corrupt correction ledger keeps its own text |

## Where these lessons go

- `specs/change/change.3md`
- `specs/cmd_change/cmd_change.3md`
