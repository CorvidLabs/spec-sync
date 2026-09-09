---
id: quote-schema-pattern-regex-as-valid-toml-in-the-full-config-fixture
state: archived
type: bug_fix
base_commit: a9169aacb0d8807c85dd5e752ebe32453d9c9b8b
---

# Quote schema_pattern regex as valid TOML in the full-config fixture

## Intent

quote schema_pattern regex as valid TOML in the full-config fixture

## Affected Canonical Specs

- `config`

## Acceptance Criteria

- cargo test --bin specsync completes without the harness exiting abnormally; test_toml_full_config loads and asserts schema_pattern is CREATE TABLE (\w+); a double-quoted illegal TOML escape in schema_pattern sets load_error via load_config_allowing_unloadable

## No-spec Rationale

Specs already require unloadable TOML to fail closed. test_toml_full_config wrote schema_pattern with an illegal basic-string escape that the line scanner accepted and toml::from_str rejects; load_config then process::exit(1) killed the unit-test harness. Quote it as a literal string so the happy-path fixture loads.
