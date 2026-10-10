# Lesson bundle — when-i-want-a-pull-request-one-command-archives-the-change-and-prepares-it

Material for folding this change's lessons into the affected specs' `context.md`.
Synthesise from what actually happened below; do not restate the change description.

## What this change was

- **Title**: When I want a pull request, one command archives the change and prepares it
- **Kind**: Feature
- **Specs**: cmd_change, cli_args
- **Paths**: src/commands/change.rs, src/cli.rs, site/src/content/docs/workflow.md, hi/ship.md, specs/cmd_change/cmd_change.spec.md, specs/cmd_change/requirements.md, specs/cmd_change/context.md, specs/cmd_change/tasks.md, specs/cmd_change/testing.md, specs/cli_args/cli_args.spec.md, specs/cli_args/requirements.md, specs/cli_args/context.md, specs/cli_args/tasks.md, specs/cli_args/testing.md
- **Acceptance**: When I want a pull request, one command archives the change and prepares the pull request.
- **Acceptance**: If the pull request is already open, that command leaves that pull request in place.
- **Acceptance**: That command does not merge the pull request.

## Evidence

- Verification commit: `8b3286cd75e0d1e311b339deabc82e45abff34b6`
- Base commit: `4deb0ff097987e0edadc92359a7d0a278579dd2d`
- Verified by: `specsync check --spec cli_args --spec cmd_change`

## From the change's context.md

# Context

`change ship` archives and commits, then tells the person to push and open the pull request themselves. That second half is where pull requests land with the change still open, or with the archive commit still local.

`change pr` does the rest: archive when the change is ready, push the branch, and open the pull request. An existing pull request for the branch is reused. The command stops before merge.

## From the change's design.md

# Design

`change pr` calls the ship path without `--push` when the change is not archived, so the archive commit happens once. It then pushes the current branch, setting `origin` as upstream when there is none. `gh pr view` reads the pull request for that branch. `gh pr create` runs only when that view fails. The printed next step is to wait for green checks and then merge. Nothing in the command calls `gh pr merge`.

## From the change's testing.md

# Testing

REQ-cmd-change-020
REQ-cli-args-019

- `change_pr_next_step_waits_for_green_checks_and_does_not_merge` in `src/commands/change.rs` pins the line the command prints after the pull request exists.
- Preparing this branch's existing pull request is the dogfood: `specsync change pr` prints that URL and does not open a second pull request.

## Where these lessons go

- `specs/cmd_change/context.md`
- `specs/cli_args/context.md`
