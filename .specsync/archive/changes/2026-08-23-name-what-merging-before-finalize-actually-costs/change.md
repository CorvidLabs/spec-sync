---
id: name-what-merging-before-finalize-actually-costs
state: archived
type: bug_fix
base_commit: e6afd395d7fa4ece945f4c1a30e1062a52a75a68
---

# Name what merging before finalize actually costs

## Intent

name what merging before finalize actually costs

## Affected Canonical Specs

- `cmd_change`
- `cli_args`

## Acceptance Criteria

- The merge-before-finalize warning names the cost that lands on other changes, not only the one being merged: it states that merging first blocks every earlier accepted change sharing a delivery input from archiving until this one is finalized or those are reopened. The wording is pinned by a test on a pure function, so a later refactor cannot silently shrink it back. All four sites carry the corrected cost, including the CLI help text.

## No-spec Rationale

diagnostic wording within existing module contracts; the warning gains the second-order cost it always had, extracted to a pure function so it can be pinned; no spec text changes
