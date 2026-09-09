## ADDED

### REQUIREMENT REQ-cmd-generate-002

`specsync generate` SHALL fail closed when it skips every unspecced module because no source files were found, instead of exiting 0 with an empty write set.

Acceptance Criteria
- Text mode lists the skipped module names and exits 1.
- JSON mode includes `skipped_no_files` and exits 1 when `generated` is empty and that list is not.
- Batch generate (`--batch`) uses the same rule.
- A fully covered project still prints `No specs to generate` and exits 0.
