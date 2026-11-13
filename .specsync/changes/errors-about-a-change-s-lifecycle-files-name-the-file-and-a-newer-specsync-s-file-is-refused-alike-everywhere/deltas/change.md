---
change: errors-about-a-change-s-lifecycle-files-name-the-file-and-a-newer-specsync-s-file-is-refused-alike-everywhere
artifact: delta
module: change
---

## ADDED

### REQUIREMENT REQ-change-164

Every error SpecSync reports while reading one of a change's lifecycle files (`state.json`,
`approvals.json`, `verification.json`, `verification-attempts.json`, `review.json`,
`review-attempts.json`, `corrections.json`, `finalization.json` or `accepted-state.json`) SHALL
name that file's path, whatever the cause: the file cannot be read, is not JSON, holds a value of
the wrong type, or lacks a field (#434).

#### Acceptance Criteria

- A type error in `approvals.json`, for example `"approvals": "soon"`, is reported by
  `change status`, `change audit` and `change check` with the path of that `approvals.json`.
- The same holds for each lifecycle file listed above.
- An error that already named its file names it once, not twice.

### REQUIREMENT REQ-change-165

A lifecycle file whose `schema_version` is newer than this SpecSync supports SHALL be refused the
same way wherever it is read, with one message:
`<path> was written by a newer SpecSync (schema version <n>); upgrade specsync to read it`.
`change status`, `change show`, `change audit` and every lifecycle verb SHALL refuse it, and
`change audit` SHALL NOT report such a change valid (#434).

#### Acceptance Criteria

- `state.json` with `schema_version: 99` is refused with that message, not read silently.
- `verification-attempts.json` with `schema_version: 99` is refused by `change audit` and
  `change status`, not only by `change check`.
- A lifecycle file at a schema version this SpecSync supports reads exactly as before.
