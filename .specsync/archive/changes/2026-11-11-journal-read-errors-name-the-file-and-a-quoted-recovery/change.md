---
id: journal-read-errors-name-the-file-and-a-quoted-recovery
state: archived
type: bug_fix
base_commit: 2321597cf803f495c7491901b088a040fe42a457
---

# Journal read errors name the file and a quoted recovery

## Intent

journal read errors name the file and a quoted recovery

## Affected Canonical Specs

- `change`

## Acceptance Criteria

- a non-UTF-8 or unreadable journal gets the same path+recovery error as an unparseable one; the printed recovery works verbatim in a project whose path contains a space

## No-spec Rationale

Not applicable
