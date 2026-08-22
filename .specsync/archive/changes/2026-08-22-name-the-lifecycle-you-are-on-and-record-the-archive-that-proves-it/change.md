---
id: name-the-lifecycle-you-are-on-and-record-the-archive-that-proves-it
state: archived
type: bug_fix
base_commit: 4e86140199762e66900b26bbc4f1ea3177bdfde3
---

# Name the lifecycle you are on and record the archive that proves it

## Intent

name the lifecycle you are on and record the archive that proves it

## Affected Canonical Specs

- `change`
- `cmd_change`
- `cmd_init`

## Acceptance Criteria

- A squash-merged workflow-v2 archive is recognised as recorded on the default branch: measured across this repository's 172 archives, the anchor predicate moves from 71 anchored to 172, with zero remaining unanchored and no archive moving from valid to invalid. A repository still on the legacy policy names that fact at both entry points (init and change new) while a workflow-v2 repository stays completely silent. The discriminating test fails on a binary built from a separate checkout at 4e861401 and its control passes on both.

## No-spec Rationale

behaviour and diagnostics only within existing module contracts; no spec text changes
