## ADDED

### REQUIREMENT REQ-cmd-change-025

The change CLI SHALL expose historical reconciliation as an opt-in scope request and disclose that historical file transitions remain unproven in status/show and finalize output without altering normal output.

#### Acceptance Criteria

- The command delegates all validation to the domain and never approves, verifies, archives or merges.
- JSON carries explicit unproven status for the new mode; text uses plain disclosure without printing digest-bearing internals.
