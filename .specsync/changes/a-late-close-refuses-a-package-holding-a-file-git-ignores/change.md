---
id: a-late-close-refuses-a-package-holding-a-file-git-ignores
state: draft
type: bug_fix
base_commit: 18385ef5ead84f0b51d70a29a6fed8a3e5ca222a
---

# A late close refuses a package holding a file Git ignores

## Intent

A late close refuses a package holding a file Git ignores

## Affected Canonical Specs

- `change`

## Acceptance Criteria

- doctor --fix --close-late refuses a merged package that holds a file Git ignores, naming that the package differs from the remote default, and writes nothing (red then green)

## No-spec Rationale

Makes the late close match REQ-change-143: it moves a package that is committed on the remote default branch, byte for byte. The untracked-file check passed --exclude-standard, so a file Git ignores inside the package, never committed, did not block it and would be moved too.
