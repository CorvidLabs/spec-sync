## ADDED

### REQUIREMENT REQ-registry-005

`generate_registry` SHALL find spec files only through the confined spec walk (`find_spec_files_checked`), so a `specs` link that leaves the project is never walked.

Acceptance Criteria
- `tests/integration/project_reads.rs` and the walk change keep `generate_registry` on the confined walk.
