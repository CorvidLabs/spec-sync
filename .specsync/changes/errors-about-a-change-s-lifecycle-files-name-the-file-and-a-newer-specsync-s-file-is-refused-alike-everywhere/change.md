---
id: errors-about-a-change-s-lifecycle-files-name-the-file-and-a-newer-specsync-s-file-is-refused-alike-everywhere
state: approved
type: feature
base_commit: 18385ef5ead84f0b51d70a29a6fed8a3e5ca222a
---

# Errors about a change's lifecycle files name the file, and a newer SpecSync's file is refused alike everywhere

## Intent

Errors about a change's lifecycle files name the file, and a newer SpecSync's file is refused alike everywhere

## Affected Canonical Specs

- `change`

## Acceptance Criteria

- a type error in approvals.json, or any lifecycle file, is reported with that file's path by change status, change audit and change check; a state.json or verification-attempts.json with a newer schema_version is refused by change status and change audit with the one newer-SpecSync message naming the file; files at supported versions read as before

## No-spec Rationale

Not applicable
