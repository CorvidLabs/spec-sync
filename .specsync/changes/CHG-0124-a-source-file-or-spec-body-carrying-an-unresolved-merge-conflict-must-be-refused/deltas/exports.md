## ADDED

### REQUIREMENT REQ-exports-007

Extraction SHALL report a scan that unioned both sides of a conflict hunk as conflicted.

Acceptance Criteria
- The result carries which side contributed which symbols, so a diagnostic can name the mechanism rather than announce that markers exist.
- A conflicted scan is never returned as an ordinary symbol list.
- Detection requires declarations on BOTH sides, so a complete conflict triple inside a string literal does not qualify.
