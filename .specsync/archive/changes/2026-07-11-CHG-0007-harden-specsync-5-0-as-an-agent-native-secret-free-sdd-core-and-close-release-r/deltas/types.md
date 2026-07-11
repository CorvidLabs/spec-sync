## ADDED

### REQUIREMENT REQ-types-001
Core shared types SHALL contain no embedded inference-provider or credential configuration.

Acceptance Criteria
- `AiProvider` and its helper API are removed.
- `SpecSyncConfig` has no AI provider, model, command, key, base URL, or timeout fields.
