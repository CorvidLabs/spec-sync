## ADDED

### REQUIREMENT REQ-config-010

The default configuration loader SHALL refuse an unloadable config file.

Acceptance Criteria
- A config file that exists and cannot be used stops the command, whichever loader it reached.
- A permissive loader remains available under a name that states it bypasses the refusal, so the bypass is deliberate rather than forgotten.
- A project with no config file at all is unaffected; the built-in defaults remain a legitimate run.
