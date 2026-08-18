---
change: CHG-0144-a-staleness-answer-must-not-read-an-unreadable-source-as-freshness
artifact: context
---

# Context

Found by dogfooding the 6.0 candidate. A spec whose only `files:` entry had been
deleted and committed:

    $ specsync stale
    Result:    0/1 specs are stale
    ✓ All specs are up to date with their source files.
    rc=0

    $ specsync check          # same tree
    rc=1

Two commands, one tree, opposite verdicts — and the wrong one is the one that
sounds reassuring. The comparison had nothing to compare, and that absence was
reported as freshness. It is this release's organizing defect landing on the one
command whose entire job is to answer this question.

## A deletion is not an absence of evidence

Git can state it:

    git rev-list --count <spec-commit>..HEAD -- src/invoice.rs   ->  1
    git log --diff-filter=D -- src/invoice.rs                    ->  deleted in 7849a74

So a deletion is a DEFINITE fact, worse than drift rather than softer. That also
rules out the obvious repair: the deletion measures ONE commit against a default
threshold of FIVE, so simply removing the guard leaves the spec "fresh" and the
reported bug intact.

## Five sites, three disguises

The belief was shared, and each site expressed it differently:

| site | disguise |
|---|---|
| `stale` / `report` / `check` | `.exists()` guard then a bare `continue` |
| `scoring` | skips, then reports the git half MEASURED at zero |
| `lifecycle` | no guard at all — the deletion counts as one commit and the threshold buries it |

`scoring` looked correct from outside because a separate file-existence criterion
does penalise the missing file; only the drift half lied.

## The vocabulary already existed

`report` models staleness as an optional and already counts unmeasured modules.
`scoring` already has a withheld verdict for the missing-history case.
`ExportScan` already separates unreadable from empty. Each of these sites simply
was not wired to the concept its own module already had.
