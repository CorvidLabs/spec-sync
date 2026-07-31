## ADDED

### REQUIREMENT REQ-commands-005

Command orchestration SHALL preserve fallible schema validation and visible ignore suppression
without reporting false success.

Acceptance Criteria

- A schema snapshot failure is returned as an error and cannot become an empty successful comparison.
- Text and structured check/report outputs distinguish emitted warnings from suppressed warnings.
- Suppression details are deterministic across text, JSON, Markdown, and GitHub formats.
- Existing notice, strict, coverage, and exit semantics remain compatible except where a prior path
  falsely reported success.
