## ADDED

### REQUIREMENT REQ-config-011

The configuration scanner SHALL report an unterminated header as a load failure.

Acceptance Criteria
- A line that opens a header and never closes it is recorded rather than silently skipped.
- The refusal uses the same wording as the unreadable-file shape, so a consumer matching a refusal need not know which door produced it.
- Valid TOML the scanner does not implement is not rejected; the test is limited to the unambiguous unterminated-header case.
