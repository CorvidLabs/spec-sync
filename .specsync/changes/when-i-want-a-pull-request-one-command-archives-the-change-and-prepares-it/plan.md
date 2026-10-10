---
change: when-i-want-a-pull-request-one-command-archives-the-change-and-prepares-it
artifact: plan
---

# Plan

## Approach

Add `specsync change pr`. When the change is still open and ready, it runs the same archive-and-commit path as `change ship`. It then pushes the current branch and runs `gh pr view`. If that pull request exists, it prints the URL. If not, it runs `gh pr create` against the repository's default branch. The next line says to wait until the checks are green and then merge. The command never merges.

## Out of scope

No merge. No new host for pull requests besides `gh`. No change to the Archive check's sentence. A change that is not ready is still refused with one command.

## Steps

1. Add the `pr` subcommand and `run_prepare_pr`.
2. Push with an upstream when the branch does not have one.
3. Reuse an open pull request for the branch.
4. Say that merging waits on green checks.

## Risks

`gh` might be missing or unauthenticated. The command fails with gh's own error and leaves the archive commit in place.

Pushing a branch that already has an open pull request updates that pull request. It does not open a second one.

## Constraints consulted

none
