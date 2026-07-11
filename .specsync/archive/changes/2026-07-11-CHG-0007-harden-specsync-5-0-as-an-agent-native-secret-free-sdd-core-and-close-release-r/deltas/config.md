## ADDED

### REQUIREMENT REQ-config-001
Configuration loading SHALL never interpret repository or local configuration as inference credentials or executable AI commands.

Acceptance Criteria
- AI configuration fields are removed from JSON/TOML readers and writers.
- Legacy AI keys produce migration guidance without activating behavior.
- The obsolete AI-only local override merge is removed.
