## ADDED

### REQUIREMENT REQ-cmd-generate-001
The generate command SHALL scaffold specs deterministically without invoking inference, network providers, credentials, or shell commands.

Acceptance Criteria
- Provider and model flags are absent.
- Default, batch, uncovered, and JSON modes retain their non-AI behavior.
- Generated paths and exit status remain machine-readable for coding agents.
