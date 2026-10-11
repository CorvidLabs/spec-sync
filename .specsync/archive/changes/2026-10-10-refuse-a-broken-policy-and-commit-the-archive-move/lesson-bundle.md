# Lesson bundle — refuse-a-broken-policy-and-commit-the-archive-move

Material for folding this change's lessons into each affected module's lesson file.
Synthesise from what actually happened below; do not restate the change description.

## What this change was

- **Title**: Refuse a broken policy and commit the archive move
- **Kind**: BugFix
- **Specs**: change, cmd_change, cmd_rules
- **Paths**: src/change.rs, src/change_tests.rs, src/commands/change.rs, src/commands/rules.rs, specs/change/change.spec.md, specs/change/requirements.md, specs/change/context.md, specs/change/testing.md, specs/change/tasks.md, specs/cmd_change/cmd_change.spec.md, specs/cmd_change/requirements.md, specs/cmd_change/context.md, specs/cmd_change/testing.md, specs/cmd_change/tasks.md, specs/cmd_rules/cmd_rules.spec.md, specs/cmd_rules/requirements.md, specs/cmd_rules/context.md, specs/cmd_rules/testing.md, specs/cmd_rules/tasks.md, site/src/content/docs/workflow.md, site/src/content/docs/quickstart.md, site/src/content/docs/cli.md, site/src/content/docs/configuration.md
- **Acceptance**: Opening a change and asking its status fail when .specsync/sdd.json cannot be parsed.
- **Acceptance**: specsync rules warns when the named principles file is not on disk.
- **Acceptance**: change finalize commits the archive move.

## Evidence

- Verification commit: `3fb7313b333132e9f49e337a7d769e85ec16d588`
- Base commit: `4d42cd401623481ae6b99e12142b7aa8767fc25e`
- Verified by: `specsync check --spec change --spec cmd_change --spec cmd_rules`

## From the change's context.md

# Context

Three review notes were left until after the optional module 3md file landed. A broken `.specsync/sdd.json` was swallowed by `change new` and `change status`, so a damaged policy looked like a project with no principles. `specsync rules` named a principles file that was not on disk and said nothing was wrong. `change finalize` moved the package to the archive and left that move uncommitted, so the next checkout could lose it. `change ship` already committed the same move.

The constraint this change honors is that a broken policy must not look like no principles file. SpecSync still does not copy lessons into the principles file, and it still does not merge the pull request.

## From the change's testing.md

# Testing

## Requirement evidence

| Requirement | Evidence |
|-------------|----------|
| REQ-change-108 | `change_new_refuses_a_broken_sdd_policy` writes `{` to `.specsync/sdd.json` and requires `create_change` to fail with `invalid SDD policy`. |
| REQ-cmd-change-021 | `status_reads_the_same_broken_policy_error` requires `configured_principles_file` to return that same error. `change status` returns it before printing a briefing. |
| REQ-cmd-change-022 | `finalize_commits_the_archive_move` requires `git log -1` to be `chore(lifecycle): archive <id>` and porcelain to drop `.specsync/changes/<id>`. |
| REQ-cmd-rules-003 | `rules_says_when_the_named_principles_file_is_missing` requires the summary to name the path, say missing, and say to create the file or clear `principles_file`. |

- `cargo test --bin specsync -- change_new_refuses_a_broken_sdd_policy` passed.
- `cargo test --bin specsync -- status_reads_the_same_broken_policy_error` passed.
- `cargo test --bin specsync -- rules_says_when_the_named_principles_file_is_missing` passed.
- `cargo test --bin specsync -- finalize_commits_the_archive_move` passed.

Opening a change and asking its status fail when `.specsync/sdd.json` cannot be parsed. `specsync rules` warns when the named principles file is not on disk. `change finalize` commits the archive move.

## Where these lessons go

- `specs/change/context.md`
- `specs/cmd_change/context.md`
- `specs/cmd_rules/context.md`
