## ADDED

### REQUIREMENT REQ-validator-052

`compute_coverage_checked` SHALL read each retained spec, to count the files it names that are missing, only through the bounded project read (8 MiB, REQ-util-003); a refused read SHALL be skipped as a missing spec is.

Acceptance Criteria
- `tests/integration/project_reads.rs` `no_project_file_is_read_outside_the_bounded_read`, scanning `src/validator/tests.rs`, finds no direct read there
