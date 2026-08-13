## ADDED

### REQUIREMENT REQ-cmd-check-008

An automatic fix that could not be applied SHALL be reported, and SHALL NOT be reported as
success.

Acceptance Criteria
- A spec that cannot be written is reported with its path and the underlying error, and the command exits non-zero.
- A spec that cannot be read is reported the same way rather than skipped silently.
- Failures are reported in every output format, so a machine consumer that requested a mutation is not left reading only a success payload.
- A writable spec is still repaired and the command still exits zero.
- A dry run attempts no write and therefore reports no write failure, exiting zero even when the target is not writable.
