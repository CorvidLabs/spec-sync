## ADDED

### REQUIREMENT REQ-cmd-check-014

`check` SHALL disclose cited files whose drift it could not measure, including on specs where other files were measured.

Acceptance Criteria
- A cited file that is absent or is a directory is named as unmeasurable rather than skipped without record.
- The disclosure appears even when the spec's other files produced a drift number, so that number is not read as covering the whole spec.
