# Lesson bundle: audit-canonical-requirements-lost-after-the-last-change-is-archived

Material for folding this change's lessons into each affected module's lesson file.
Synthesise from what actually happened below; do not restate the change description.

## What this change was

- **Title**: Audit canonical requirements lost after the last change is archived
- **Kind**: BugFix
- **Specs**: change, cmd_check
- **Paths**: src/change/vanished_requirements.rs, src/change/historical_definition_digest_matches.rs, src/commands/check.rs, tests/integration/vanished_requirements.rs, specs/change/, specs/cmd_check/
- **Acceptance**: With no active changes on an SDD-enabled Git branch, both change audit and check --strict report each unexplained canonical requirement loss, its original file and a concrete restore command. Managed CI may run only strict check. Both commands pass on a separate no-loss control. A Git project that never enabled SDD keeps its usual strict drift checks. Approved branch removals and requirements moved under the same ID pass. A missing comparison ref is disclosed and incomplete comparison history gives fetch guidance.

## Evidence

- Verification commit: `a9328f4df3e0e39a10d2262f6e3eb97cf82efc4e`
- Base commit: `5314fb6ae13f7b9cc656cc2aef0fb875a59b2026`
- Verified by: `specsync check --spec change --spec cmd_check`

## From the change's context.md

# Context

An archive-first CI branch with 442 archived packages and no active workspaces lost REQ-cmd-change-040. Both strict audit modes returned empty errors and warnings. See the #1105 reproducer and its separate no-loss control. This draft is stacked on #1108 at 5314fb6a for its bounded historical requirement reader.

The maintainer answered the #1105 interview with “Both `change audit` and `check --strict`, so projects running either command catch the loss”. HI records the answer as CHANGE-17.d under the existing requirement-preservation intent.

The required Lifecycle gate already runs `change audit --strict` on full history after Archive. A project can use the SpecSync lifecycle while running only `check --strict` in CI. Both gates should call the same branch comparison and report the same lost IDs and restore guidance. Projects that have never adopted the lifecycle retain ordinary strict drift checking.

## From the change's testing.md

# Testing

Use a real Git comparison base and a branch with zero active workspaces. In the loss fixture, archive the final change and then remove an unrelated canonical requirement. Run `change audit`, `change audit --strict`, `change audit --archives`, and `check --strict`. Compare failed exit status, missing ID, original path and the `git show` restore command. Include JSON output and require each command to report the missing ID once. Check the Git worktree and history stay unchanged.

Use a separate no-loss control from the same base. Keep the canonical requirement and archive the corresponding change. Both command families pass. Separately exercise a tracked move, a valid removal archived on this branch, and a prior archive that mentions the missing ID. Run strict check alone on the managed archive-only branch to show it catches the loss without an audit invocation. In a separate Git project that has never enabled SDD, use the same kind of deletion and a readable default comparison ref to show strict drift validation does not require a removal delta. Keep the ordinary `check` control for unrelated corrupt workspace evidence.

In a checkout with a known default ref but no readable merge base, require a refusal with fetch guidance. In a checkout without a usable default ref, require an explicit skipped comparison and preserve the remaining checks.

## Requirement evidence

| Requirement | Evidence |
| --- | --- |
| REQ-change-archived-requirement-loss-001 | Planned CLI loss, no-loss, branch removal, and incomplete history cases in tests/integration/vanished_requirements.rs |
| REQ-cmd-check-004 | Planned strict-check loss and no-loss checks using the same fixtures, including strict-check-only CI, an SDD-opt-out control and the plain-check control |

## Where these lessons go

- `specs/change/change.3md`
- `specs/cmd_check/cmd_check.3md`
