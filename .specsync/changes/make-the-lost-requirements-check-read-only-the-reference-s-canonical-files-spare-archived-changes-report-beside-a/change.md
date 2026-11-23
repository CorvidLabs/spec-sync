---
id: make-the-lost-requirements-check-read-only-the-reference-s-canonical-files-spare-archived-changes-report-beside-a
state: verifying
type: bug_fix
base_commit: ed258f58d734be7c4c0ac092ddbc934f492630d3
---

# Make the lost-requirements check read only the reference's canonical files, spare archived changes, report beside a change's own errors, and name an unreadable path

## Intent

Make the lost-requirements check read only the reference's canonical files, spare archived changes, report beside a change's own errors, and name an unreadable path

## Affected Canonical Specs

- `change`
- `cmd_change`

## Acceptance Criteria

- (1) REQ-change-175: the files read at the reference come only from what was canonical there; a branch that registers a spec beside, or moves specs_dir onto, a requirements.md that was already there and rewrites it is not refused for its old ids; a requirement moved into a newly canonical file is found there. (2) REQ-change-176: change audit reports the lost requirements for each active change whether or not its approval, delta files and delta bodies validate, beside its own error. (3) REQ-change-177: when a requirement would be reported lost and a requirements directory or file in the working tree could not be read, the check says it cannot tell, naming each path and the error and the way out; nothing changes when nothing is lost. (4) REQ-cmd-change-045: change ship-status lists the lost-requirements blocker only for a change that can still be checked, finalized or shipped, not for an archived or retired one.

## No-spec Rationale

Not applicable
