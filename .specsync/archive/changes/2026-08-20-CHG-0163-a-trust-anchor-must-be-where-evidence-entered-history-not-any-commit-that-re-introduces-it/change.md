---
id: CHG-0163-a-trust-anchor-must-be-where-evidence-entered-history-not-any-commit-that-re-introduces-it
state: archived
type: bug_fix
base_commit: 65755ac7e27693ae88ea39fd0c681ecc1949b412
---

# A trust anchor must be where evidence entered history, not any commit that re-introduces it

## Intent

a trust anchor must be where evidence entered history, not any commit that re-introduces it

## Affected Canonical Specs

- `change`

## Acceptance Criteria

- authenticated_accepted_transition authenticates an archived change by finding a commit that ADDED its accepted-state.json whose committed evidence bytes equal the working-tree bytes. There is no cutoff, no ancestry bound and no ordering rule, so ANY commit that re-introduces the package becomes a valid anchor for whatever is on disk at the time. Because --diff-filter=A matches only additions, a commit that merely tampers is never an anchor and is correctly refused; a commit that re-introduces the package is, and necessarily carries the tampering with it. Three shapes exploit this, measured against origin/main: tamper then git mv the archive directory; tamper and relocate in one commit; and a forged reopen and re-archive pair in which the directory keeps its name throughout and the laundering happens at the active workspace path. 117 of 161 archived packages route through this path. Done when: an anchor must be the earliest reachable commit that introduced this change's package, identified by the ID inside the committed state.json rather than by directory name; the same bound applies to the active-workspace stages and the working-tree fallback; all three laundering shapes are refused; an honest relocation still authenticates; and every archive that authenticates today still does.

## No-spec Rationale

Not applicable
