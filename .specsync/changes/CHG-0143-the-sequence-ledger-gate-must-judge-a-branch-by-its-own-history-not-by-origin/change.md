---
id: CHG-0143-the-sequence-ledger-gate-must-judge-a-branch-by-its-own-history-not-by-origin
state: implementing
type: bug_fix
base_commit: 06c1bcca162750d5761b0e530b9da52ce9995c53
---

# The sequence ledger gate must judge a branch by its own history, not by origin

## Intent

the sequence ledger gate must judge a branch by its own history, not by origin

## Affected Canonical Specs

- `change`

## Acceptance Criteria

- a branch merely behind the default branch can still run change new and allocates past origin's high-water mark rather than onto it; a branch that lowered the ledger below the highest mark it itself recorded is still refused, including when it raised the ledger first and the rewrite stays above the point it diverged; the gate needs no remote and does not silently disable itself when origin is absent; the error names the mark that was lost and the recovery command that applies

## No-spec Rationale

fixes a regression shipped in adbfb442: the read-side gate compared the ledger against origin/main, so any branch merely behind origin was refused with a message telling the operator to restore a file that was not corrupt
