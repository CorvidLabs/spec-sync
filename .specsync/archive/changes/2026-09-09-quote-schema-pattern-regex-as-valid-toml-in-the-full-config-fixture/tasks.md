---
change: quote-schema-pattern-regex-as-valid-toml-in-the-full-config-fixture
artifact: tasks
---

# Tasks

- [x] Quote `schema_pattern` as a TOML literal in `test_toml_full_config` and assert the regex
- [x] Add `test_toml_invalid_schema_pattern_escape_sets_load_error` using `load_config_allowing_unloadable`
- [x] Re-run the full `--bin specsync` suite
