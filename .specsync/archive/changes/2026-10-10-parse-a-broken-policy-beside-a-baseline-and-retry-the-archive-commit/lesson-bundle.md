# Lesson bundle — parse-a-broken-policy-beside-a-baseline-and-retry-the-archive-commit

Material for folding this change's lessons into each affected module's lesson file.
Synthesise from what actually happened below; do not restate the change description.

## What this change was

- **Title**: Parse a broken policy beside a baseline and retry the archive commit
- **Kind**: BugFix
- **Specs**: change, cmd_change, cmd_rules
- **Paths**: src/change.rs, src/change_tests.rs, src/commands/change.rs, src/commands/rules.rs, tests/integration/change.rs, specs/change/change.spec.md, specs/change/requirements.md, specs/change/context.md, specs/change/testing.md, specs/cmd_change/cmd_change.spec.md, specs/cmd_change/requirements.md, specs/cmd_change/context.md, specs/cmd_change/testing.md, specs/cmd_rules/cmd_rules.spec.md, specs/cmd_rules/requirements.md, specs/cmd_rules/context.md, specs/cmd_rules/testing.md
- **Acceptance**: Opening a change fails when .specsync/sdd.json cannot be parsed, including when a workflow-v2 baseline exists.
- **Acceptance**: change finalize commits an archive that was left uncommitted, and its next action says to push that commit before merging.
- **Acceptance**: specsync rules does not treat an escaping principles path as a usable file.

## Evidence

- Verification commit: `ad1dcc4d0679df8b6a6f9007df0a09b04b4e4d92`
- Base commit: `9f4b72dd5c728f129c47a74493051b84d9d475e2`
- Verified by: `specsync check --spec change --spec cmd_change --spec cmd_rules`

## From the change's context.md

# Context

Review on pull request 807. An adopted project already has `.specsync/workflow-v2-baseline.json`, and `change new` returned workflow version 2 without parsing `.specsync/sdd.json`. A broken policy still opened a change.

`change finalize` moved the package and then committed. If the commit failed, the change was already archived, and running finalize again refused the state instead of committing the package. The next line still said only to merge.

`specsync rules` used `Path::join` for the principles file, so an absolute path or `..` could be reported as present. Approval rejects those paths.

## From the change's testing.md

# Testing

## Requirement evidence

| Requirement | Evidence |
|-------------|----------|
| REQ-change-108 | `change_new_refuses_a_broken_policy_when_the_v2_baseline_exists` writes a workflow-v2 baseline and `{` as the policy, and requires `create_change` to fail with `invalid SDD policy` without creating a change directory. |
| REQ-cmd-change-022 | `finalize_commits_an_archive_that_was_left_uncommitted` archives first, then runs finalize, and requires the archive commit. The integration test expects the next action to say to push the archive before merging. |
| REQ-cmd-rules-003 | `an_escaping_principles_path_is_not_a_present_file` requires `/etc/passwd` and `../outside.md` to be rejected as escaping the project. |

- `cargo test --bin specsync -- change_new_refuses_a_broken_policy_when_the_v2_baseline_exists` passed.
- `cargo test --bin specsync -- finalize_commits_an_archive_that_was_left_uncommitted` passed.
- `cargo test --bin specsync -- an_escaping_principles_path_is_not_a_present_file` passed.

## Where these lessons go

- `specs/change/context.md`
- `specs/cmd_change/context.md`
- `specs/cmd_rules/context.md`
