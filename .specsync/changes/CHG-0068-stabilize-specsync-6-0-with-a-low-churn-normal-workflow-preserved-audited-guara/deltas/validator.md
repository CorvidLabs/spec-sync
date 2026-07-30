## ADDED

### REQUIREMENT REQ-validator-009

Schema-aware validation SHALL compare canonical identities from the command invocation's checked
schema snapshot and SHALL never pass vacuously when schema validation was requested.

Acceptance Criteria

- Quoted, qualified, and mixed-case table declarations compare through one canonical identity.
- An unqualified declaration may match a unique qualified table leaf; a qualified declaration
  requires the full identity.
- Invalid or captureless `schema_pattern` configuration is visible and cannot silently erase schema
  validation.
- Declared `db_tables` without a configured readable schema produce a finding instead of a vacuous
  pass.
