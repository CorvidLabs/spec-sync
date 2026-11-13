---
id: change-correct-reads-its-summary-inside-a-change-read-scope
state: implementing
type: bug_fix
base_commit: 18385ef5ead84f0b51d70a29a6fed8a3e5ca222a
---

# Change correct reads its summary inside a change read scope

## Intent

change correct reads its summary inside a change read scope

## Affected Canonical Specs

- `change`

## Acceptance Criteria

- a change correct, live validation included, spawns fewer Git processes than one summary of the same change read without a scope; the corrected record, correction ledger and summary are unchanged

## No-spec Rationale

No requirement changes: change correct already returns the corrected change's summary, and only the cost of building it changes. It took that summary with no change read scope, so it got none of the memoized reads, persistent read cache or prefetch workers that the read commands use (#921)
