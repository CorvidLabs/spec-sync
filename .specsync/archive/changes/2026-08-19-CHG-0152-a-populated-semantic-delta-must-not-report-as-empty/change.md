---
id: CHG-0152-a-populated-semantic-delta-must-not-report-as-empty
state: archived
type: bug_fix
base_commit: cf38520e965a8c7d616c8a81689fcc1bfd0e4e06
---

# A populated semantic delta must not report as empty

## Intent

a populated semantic delta must not report as empty

## Affected Canonical Specs

- `change`

## Acceptance Criteria

- A semantic delta file that contains text but no recognized operation heading reports that it has no recognized operation section and names the allowed headings, instead of reporting that the file is empty; the same correction applies to the historical delta path, which carries the identical wording today. A file that is empty or whitespace-only still reports that it is empty. Item headings are accepted case-insensitively, matching the operation headings that already are, so ### requirement parses exactly as ### REQUIREMENT does; an unrecognized heading is still refused and still names the allowed values. A valid uppercase delta still approves unchanged, and every archived delta still parses to the same items, since widening acceptance cannot alter files that were already grammar-conformant when they were approved.

## No-spec Rationale

Not applicable
