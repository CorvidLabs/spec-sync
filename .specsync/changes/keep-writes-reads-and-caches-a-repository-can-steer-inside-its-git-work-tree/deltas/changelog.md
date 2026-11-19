## ADDED

### REQUIREMENT REQ-changelog-002

`changelog` SHALL ask a spec blob's size before reading it, SHALL skip a blob larger than 8 MiB with one warning that names the spec and the ref, and SHALL keep at most 8 MiB of the blob's bytes; the rest of the changelog SHALL still print.

Acceptance Criteria
- A 9 MiB spec in history is skipped with a warning naming it, and a spec added in the same range is still listed.
