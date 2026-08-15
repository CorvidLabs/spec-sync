## ADDED

### REQUIREMENT REQ-generator-004

Generated specs SHALL contain only symbols the configured surface includes.

Acceptance Criteria
- A generated spec, once activated, passes `check` without orphan-export errors.
- The tool cannot emit a spec its own validator rejects.
