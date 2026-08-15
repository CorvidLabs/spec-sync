## ADDED

### REQUIREMENT REQ-cmd-wizard-002

`wizard` SHALL keep working over a broken configuration.

Acceptance Criteria
- It requests the permissive loader explicitly, because a command whose job is repairing the config must run on the project that needs repairing.
