## ADDED

### REQUIREMENT REQ-scoring-004

The API dimension SHALL grade against the configured export surface.

Acceptance Criteria
- `score` and `check` never disagree about which symbols constitute a module's API.
- A symbol outside the configured surface is neither counted nor named as undocumented.
- A project on the default surface scores exactly as before.
