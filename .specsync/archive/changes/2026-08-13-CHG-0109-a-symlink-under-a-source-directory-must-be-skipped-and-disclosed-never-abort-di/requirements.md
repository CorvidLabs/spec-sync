---
change: CHG-0109-a-symlink-under-a-source-directory-must-be-skipped-and-disclosed-never-abort-di
artifact: requirements
---

# Requirements

## REQ-validator-012

Source discovery SHALL skip a symlinked entry rather than failing the traversal.

Acceptance Criteria
- A symlinked entry met during source detection or the coverage walk is recorded and skipped, and discovery continues.
- The entry is never traversed, so content beyond a link that leaves the project root is never read.
- A symlink that is a configured source directory entry remains a hard failure, because silently skipping an explicitly configured tree would drop everything the author asked to measure.
- Symlinks in the spec tree remain a hard failure.
- Recorded paths are deduplicated and normalized to forward slashes.

## REQ-types-006

The coverage report SHALL carry the symlinked entries that discovery skipped.

Acceptance Criteria
- Skipped entries are reported in a deterministic order.
- An inconclusive coverage result reports no skipped entries rather than omitting the field.

## REQ-output-003

Coverage output SHALL disclose skipped symlinked entries alongside the coverage figures.

Acceptance Criteria
- Text output names the skipped entries immediately after the coverage lines.
- Markdown output names them within the coverage section.
- A fixed number of entries are named explicitly and any remainder is summarized with a count.
- Output with no skipped entries is unchanged.

## REQ-cmd-check-006

Machine-readable check output SHALL carry the skipped symlinked entries.

Acceptance Criteria
- The JSON payload includes the full list of skipped entries, not a truncated summary.
- The field is present whenever the payload reports a result.

## REQ-commands-007

Strict validation SHALL refuse to report success for a tree whose coverage excluded skipped symlinked entries.

Acceptance Criteria
- Strict mode exits non-zero when any entry was skipped, naming how many.
- Bare validation continues to exit zero and only reports the exclusion.
- Both the text and machine-readable exit paths apply the same rule.
