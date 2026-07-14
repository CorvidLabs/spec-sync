## ADDED

### REQUIREMENT REQ-config-004

Configuration SHALL provide a default-false `require_draft_files` option, named `requireDraftFiles` in legacy JSON, that requires every draft mapping to exist when enabled.

Acceptance Criteria

- Omitted and explicit-false values preserve planned draft mappings.
- Canonical TOML reads and emits `require_draft_files = true` without losing the value during migration.
- Legacy JSON reads `requireDraftFiles` and recognizes it as a supported key.
