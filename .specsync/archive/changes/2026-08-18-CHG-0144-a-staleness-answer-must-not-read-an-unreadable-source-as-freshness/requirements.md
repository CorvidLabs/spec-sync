---
change: CHG-0144-a-staleness-answer-must-not-read-an-unreadable-source-as-freshness
artifact: requirements
---

# Requirements

## REQ-cmd-stale-004

`stale` must not report a spec as current when a file it cites no longer exists,
and must not claim an all-clear over any spec it could not measure, in any output
format.

## REQ-cmd-report-005

`report` must decline to state a staleness it could not measure, and its
run-level inconclusive flag must be set whenever any module was unmeasured.

## REQ-cmd-check-014

`check` must disclose cited files whose drift it could not measure, including on
specs where other files were measured.

## REQ-scoring-006

The git half of the freshness dimension must report itself withheld when a cited
file was deleted, rather than a measured zero, and must not apply a second
penalty for a deletion the file-existence criterion already charges for.

## REQ-cmd-lifecycle-004

The no-stale guard must fail when a spec cites a source file that no longer
exists, regardless of the configured threshold.

## REQ-git-utils-004

A shared predicate must answer whether a cited path was known to git at a given
commit and is now absent, resolved relative to the project root, and every
staleness consumer must use it rather than re-deriving the distinction.
