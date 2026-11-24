# Lesson bundle: pass-safe-directory-to-specsync-s-isolated-git-when-the-user-s-own-git-trusts-a-repository-owned-by-another-user

Material for folding this change's lessons into each affected module's lesson file.
Synthesise from what actually happened below; do not restate the change description.

## What this change was

- **Title**: Pass safe.directory to SpecSync's isolated Git when the user's own Git trusts a repository owned by another user
- **Kind**: BugFix
- **Specs**: change
- **Paths**: src/change/consider_accepted_evidence_anchor.rs, src/change/validate_loaded_change.rs, src/change_tests.rs, src/change_tests/git_environments.rs, tests/integration.rs, tests/integration/, specs/change/, site/src/content/docs/, CHANGELOG.md
- **Acceptance**: (1) A repository owned by another user, which the user's global configuration lists in safe.directory, passes change audit, change status and change check as for its owner, including from a linked worktree. (2) Without that entry it is refused naming dubious ownership, the path and git config --global --add safe.directory. (3) A repository owned by the user keeps the global configuration off, as before. (4) safe.directory = * passes the repository's own top level, never *. (5) The file is owner-only, outside the project and its Git directory, and gone when the command ends.

## Evidence

- Verification commit: `9699af6dc1face8606764cd33c380ce791ddee2d`
- Base commit: `c076583c09ff9acbe529426922a3e225eed42f56`
- Verified by: `specsync check --spec change`

## From the change's context.md

# Context

Issue #909. SpecSync's isolated Git (`configured_git_command`) sets `GIT_CONFIG_GLOBAL=/dev/null` and `GIT_CONFIG_NOSYSTEM=1`, so it drops the user's `safe.directory`. A repository owned by another user, which the user's Git trusts, is refused with "unable to inspect changed paths" or "Git repository detection failed". Measured in containers (Git 2.32 to 2.39): `-c safe.directory` and `GIT_CONFIG_COUNT` are not honoured by upstream 2.35.2 to 2.37; a `GIT_CONFIG_GLOBAL` file holding only that entry works on all of them.

Definition: ss-lead/gates/git-environments-909-1072-1077-definition.md, section 2 and REQ-change-171. Leif's choice 2 in the tester's dialog of 2026-11-20 at about 00:45Z: "Yes, temp file (Recommended)".

## From the change's design.md

# Design

- **Trigger:** only when an isolated call refuses with `detected dubious ownership`. Repository detection (`git_repository_present`) is the first isolated call a command makes, so it is the place the trust is established; the changed-path reads name the refusal too.
- **Probe:** one `git -C <root> rev-parse --show-toplevel` with the user's own environment and configuration (prompts off). Success means the user's Git trusts the repository; its output is the one path the setting names.
- **File:** a new private directory under the system temporary directory (`tempfile`, 0700) holding one file (0600, created new) with `[safe]` and `directory = <top level>` in Git's config quoting. Refused when the temporary directory lies inside the project or its Git directory. Held for the process and removed at its end.
- **Use:** `configured_git_command` passes that file as `GIT_CONFIG_GLOBAL` in place of `/dev/null` for this project; everything else stays as it is.
- **Refusal:** when the probe fails too, or the file cannot be made, the command refuses with `Git refuses this repository because it is owned by another user (detected dubious ownership at <path>). If you trust it, run \`git config --global --add safe.directory <path>\`, then run the command again.`
- **Test:** Git's `GIT_TEST_ASSUME_DIFFERENT_OWNER=1` reproduces dubious ownership for one user, on Linux and macOS.

## From the change's testing.md

# Testing

## Requirement evidence

| Requirement | Evidence |
|---|---|
| REQ-change-171 | `tests/integration/git_environments.rs`, with `GIT_TEST_ASSUME_DIFFERENT_OWNER=1`: a repository the user's global file trusts passes `change audit`, `change status` and `change check`, also from a linked worktree; without the entry it is refused naming dubious ownership, the path and the command; `safe.directory = *` passes only the top level; the file is outside the project and gone afterwards; a repository the user owns keeps the global configuration off |

## Where these lessons go

- `specs/change/change.3md`
