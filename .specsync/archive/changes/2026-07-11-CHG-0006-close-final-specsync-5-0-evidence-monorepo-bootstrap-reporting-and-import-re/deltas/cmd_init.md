## ADDED

### REQUIREMENT REQ-cmd-init-004
Initialization SHALL enable Git-dependent SDD coverage only when the project can provide Git comparison evidence.

Acceptance Criteria
- Git repositories receive normal strict SDD defaults.
- Non-Git directories initialize successfully without an immediately impossible changed-path gate.
