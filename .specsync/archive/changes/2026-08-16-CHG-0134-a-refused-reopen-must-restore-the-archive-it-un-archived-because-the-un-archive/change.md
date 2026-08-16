---
id: CHG-0134-a-refused-reopen-must-restore-the-archive-it-un-archived-because-the-un-archive
state: archived
type: bug_fix
base_commit: c977572e9adda56dfdcb1bc6b0290097ac16eb39
---

# A refused reopen must restore the archive it un-archived, because the un-archive move happens before the preconditions are checked and a correct refusal was destroying the package

## Intent

A refused reopen must restore the archive it un-archived, because the un-archive move happens before the preconditions are checked and a correct refusal was destroying the package

## Affected Canonical Specs

- `change`

## Acceptance Criteria

- A reopen that is refused leaves the dated archive package exactly where finalize wrote it, with no orphan in the active workspace and the record still archived. The refusal says the archive was restored, so a user whose reopen failed knows the package survived. Retrying reproduces the same refusal rather than a different one, because the first attempt consumed nothing. A reopen that legitimately succeeds still un-archives, so the restore cannot be passing by never moving anything. The Accepted-state reopen path, which never un-archives, is unaffected.

## No-spec Rationale

Not applicable
