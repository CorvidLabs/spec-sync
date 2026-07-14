## ADDED

### REQUIREMENT REQ-config-003

Configuration SHALL provide a default-false `include_extensionless` option (`includeExtensionless` in legacy JSON) that adds extensionless files without changing omitted or empty `source_extensions` semantics.

Acceptance Criteria

- Canonical TOML reads `include_extensionless` and emits it only when true.
- Legacy JSON reads `includeExtensionless`.
- Omitted and explicit false values preserve existing discovery.
- Omitted and empty extension lists continue to select the default supported-language set.
