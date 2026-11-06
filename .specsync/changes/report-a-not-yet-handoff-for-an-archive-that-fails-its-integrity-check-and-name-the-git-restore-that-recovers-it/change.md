---
id: report-a-not-yet-handoff-for-an-archive-that-fails-its-integrity-check-and-name-the-git-restore-that-recovers-it
state: implementing
type: bug_fix
base_commit: ad50bdef3db58a715877877d2407485bcaed0f32
---

# Report a not-yet handoff for an archive that fails its integrity check and name the git restore that recovers it

## Intent

Report a not-yet handoff for an archive that fails its integrity check and name the git restore that recovers it

## Affected Canonical Specs

- `change`
- `cmd_change`

## Acceptance Criteria

- An archived change whose terminal evidence is `corrupt-history` reports handoff readiness `not-yet`, never `safe`, in `change status --json` (`summary.handoff`) and on the text `Handoff:` line, with a reason naming the failed integrity check.
- The step before clearing, the JSON `next_action`, the text `Next:` line and `change ship-status`'s next action name the same recovery, ending with `specsync change status <id>`: `git restore --source=HEAD --staged --worktree -- <package>` when the package differs from `HEAD`; the same restore from the commit that archived it (`git log -1 --diff-filter=A --format=%H -- <package>/state.json`), then a commit, when the package matches `HEAD` but not that commit; and, when the package still matches the commit that archived it (or no package or repository can answer), `specsync change status <id> --json` to read what `terminal_evidence.reason` names and restore that from trusted Git history. No step contains a digest, and `specsync change check`, which checks nothing for an archived change, is no longer named.
- Every change ID and path interpolated into a printed command is single-quoted as one shell word, so copying the step from a repository whose change ID or archive directory carries `;`, `$`, quotes or spaces runs exactly the named command (Codex P1 on #901).
- Running the named command verbatim through a shell on a tampered, uncommitted archive, and the archiving-commit restore on committed damage, returns terminal evidence `authenticated-history` and handoff `safe` in JSON and text; `change status` writes nothing to the archive.
- An archive whose terminal evidence authenticates, including `historical-transition-unproven`, keeps handoff `safe`, the text `Next: no further action` and the JSON next action to merge the PR on GitHub if it is still open.
- Text `change status`, and the `Handoff:` line `finalize` and `ship` print, judge an archived change with the same terminal-evidence verdict JSON reports. The measured text-status cost on this repository (247 archives) is recorded in testing.md (follow-up #897), and no test, gate, bound or timeout is weakened.
- The integration test fails on main ad50bdef with readiness `safe` and passes with the fix; the quoting and outside-package tests fail on db2e3ab4 and pass with the fix.

## No-spec Rationale

Not applicable
