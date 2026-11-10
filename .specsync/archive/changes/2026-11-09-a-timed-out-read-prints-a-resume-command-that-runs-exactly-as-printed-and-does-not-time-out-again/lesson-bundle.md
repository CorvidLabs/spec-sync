# Lesson bundle: a-timed-out-read-prints-a-resume-command-that-runs-exactly-as-printed-and-does-not-time-out-again

Material for folding this change's lessons into each affected module's lesson file.
Synthesise from what actually happened below; do not restate the change description.

## What this change was

- **Title**: A timed-out read prints a resume command that runs exactly as printed and does not time out again
- **Kind**: BugFix
- **Specs**: cmd_change
- **Paths**: src/commands/change.rs, tests/integration/change.rs, specs/cmd_change, site/src/content/docs/cli.md, CHANGELOG.md
- **Acceptance**: the_resume_command_splits_back_into_the_original_arguments passes: every argument that is not made only of shell-safe characters is single-quoted, so sh splits the printed resume command back into exactly the original arguments, including $, ;, *, an apostrophe, spaces and an empty argument (committed failing first in db16db8a)
- **Acceptance**: With SPECSYNC_TIMEOUT set, the printed resume command starts with SPECSYNC_TIMEOUT= so it runs without the deadline it is resuming from (an_archived_status_past_its_timeout_exits_124_and_names_how_to_resume, extended and failing first in db16db8a)
- **Acceptance**: Without SPECSYNC_TIMEOUT the resume command is unchanged apart from quoting; --timeout and --timeout=<n> are still dropped
- **Acceptance**: The timeout report shape, exit 124 and the cache flush are unchanged

## Evidence

- Verification commit: `41b5d0c35118ab04d0d38c992166b2e542a9acda`
- Base commit: `db16db8af6a16f7ab2dd61deb2a9f2de0584dbe1`
- Verified by: `specsync check --spec cmd_change --strict`

## From the change's context.md

# Context

G-resume in Leif's burn-down (`_audits/specsync-triage-2026-11-06.md`): #923 and #953, two Codex P2s on #918 that were deferred. #918's read deadline (REQ-change-137, REQ-cmd-change-032) prints "the exact command that resumes the work". `resume_command_without_timeout` quotes an argument only when it is empty or contains whitespace, so `$`, `;`, `*` and `'` are printed bare. Pasting the command can then run something else (#923). The command also drops `--timeout` but leaves `SPECSYNC_TIMEOUT` in force, so in the same shell it times out again (#953). REQ-cmd-change-036, assigned by the orc. HI: SPEED-1.

## From the change's design.md

# Design

- **Quoting.** Every argument that is not made only of `[A-Za-z0-9_@%+=:,./-]` is single-quoted, with each `'` written as `'\''`. An empty argument becomes `''`. Plain arguments stay bare, so the common command reads as before.
- **Environment.** When `SPECSYNC_TIMEOUT` is set in the process environment, the command is prefixed with `SPECSYNC_TIMEOUT= `. An empty value parses as no deadline (`read_timeout_secs`), so the resumed run has none. Without the variable, there's no prefix.
- `--timeout <n>` and `--timeout=<n>` are still dropped. The report shape, exit 124 and the cache flush are unchanged.

## From the change's testing.md

# Testing

- `the_resume_command_splits_back_into_the_original_arguments` (unit, new; fails today because `it's` is printed bare, so `sh` can't even parse the command).
- `an_archived_status_past_its_timeout_exits_124_and_names_how_to_resume` (integration, extended; fails today: with `SPECSYNC_TIMEOUT=0` the resume command doesn't clear it).
- Unchanged: the rest of that integration test (`--timeout 0`, `--timeout=0`, the text report, byte-identical `.specsync/`), and the verify lane.

## Requirement evidence

| Requirement | Evidence |
| --- | --- |
| REQ-cmd-change-036 | unit test `the_resume_command_splits_back_into_the_original_arguments` and integration test `an_archived_status_past_its_timeout_exits_124_and_names_how_to_resume` (both failing first, `db16db8a`) |

## Where these lessons go

- `specs/cmd_change/cmd_change.3md`
