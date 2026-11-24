# Lesson bundle: download-the-history-objects-a-partial-clone-lacks-in-one-fetch-with-the-user-s-own-git-before-specsync-reads-them

Material for folding this change's lessons into each affected module's lesson file.
Synthesise from what actually happened below; do not restate the change description.

## What this change was

- **Title**: Download the history objects a partial clone lacks in one fetch with the user's own Git before SpecSync reads them
- **Kind**: BugFix
- **Specs**: change
- **Paths**: src/change/historical_definition_digest_matches.rs, src/change/read_cache.rs, src/change_tests.rs, src/change_tests/git_environments.rs, tests/integration.rs, tests/integration/, specs/change/, site/src/content/docs/, CHANGELOG.md, src/change/validate_loaded_change.rs, src/change/consider_accepted_evidence_anchor.rs, src/change/vanished_requirements.rs
- **Acceptance**: (1) In a blobless clone whose promisor remote answers only with the user's configuration, change review of a change created in an earlier commit succeeds, and the objects it read are present afterwards. (2) The download is one fetch, whatever the number of objects. (3) When the remote cannot be reached, the refusal says the clone is partial and names a git fetch command with the object ids that, run as printed, lets the same command succeed. (4) A full clone makes no fetch. (5) The fetch is one call bounded at 60 seconds and never prompts; one line on standard error says how many objects and from which remote. (6) change status, change audit and change ship-status in a partial clone fetch the same way.

## Evidence

- Verification commit: `18738bcb9ac6d1b71347f792715c945e2b3e2c7c`
- Base commit: `bdbcbd8e3ee70b89cd4934e69f8e8739073fc79e`
- Verified by: `specsync check --spec change`

## From the change's context.md

# Context

Issue #1077. In a blobless clone of a private HTTPS repository, SpecSync's isolated read of a historical object fails: `could not read Username for 'https://github.com': terminal prompts disabled` / `could not fetch <oid> from promisor remote`. The credential helper lives in the user's global or system configuration, which the isolated read switches off. SSH and public HTTPS clones read fine. One batched fetch with the user's configuration downloaded peck's 630 missing objects in 2 seconds.

Definition: ss-lead/gates/git-environments-909-1072-1077-definition.md, section 1 and REQ-change-172, option 1. Leif's choice 1 in the tester's dialog of 2026-11-20 at about 00:45Z, "Fetch it itself (Recommended)", and "yes to all 3" at about 01:00Z: every command that reads history fetches, CI included. ss-lead's note: the fetch covers #1081's reads at the reference and at a change's base commit.

## From the change's design.md

# Design

- **Detection:** `git config --get-regexp ^(extensions\.partialclone|remote\..*\.promisor)$` names the promisor remote; none means a full clone and nothing more happens.
- **Listing, isolated:** `git rev-list --objects --missing=print HEAD [<default branch>] -- .specsync <requirements paths>`; it never fetches.
- **Fetch, once per process:** `git -C <root> fetch <remote> --no-tags --no-write-fetch-head --filter=blob:none --stdin` with the ids on stdin, the user's environment and configuration, `GIT_TERMINAL_PROMPT=0`, a new session with no controlling terminal, killed after 60 s. One line on standard error first: `downloading <n> objects this partial clone lacks from \`<remote>\``.
- **Refusal:** when the fetch fails, `<n> history objects are not in this partial clone, and SpecSync could not download them from \`<remote>\`: <reason>. Download them with your own Git: \`git -C <root> fetch <remote> --no-tags --no-write-fetch-head --filter=blob:none <oid> …\`, then run the command again.`

## From the change's testing.md

# Testing

## Requirement evidence

| Requirement | Evidence |
|---|---|
| REQ-change-172 | `tests/integration/git_environments.rs`, with a blobless clone whose remote resolves only through the user's global `insteadOf`: `change review`, `status`, `audit` and `ship-status` succeed after one fetch with the stderr line, and the objects are present; with the user's configuration gone, the refusal names a `git fetch … <oid> …` that, run as printed, lets the command succeed; a full clone makes no fetch; a blobless clone on main with an active change audits #1081's base-commit requirements after the one fetch |

## Where these lessons go

- `specs/change/change.3md`
