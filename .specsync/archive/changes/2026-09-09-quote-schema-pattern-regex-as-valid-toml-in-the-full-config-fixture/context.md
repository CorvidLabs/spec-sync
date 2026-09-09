---
change: quote-schema-pattern-regex-as-valid-toml-in-the-full-config-fixture
artifact: context
---

# Context

`cargo test --bin specsync` dies after the last `config::` companions test with "test exited abnormally" and no FAILED row. `--nocapture` shows:

```
schema_pattern = "CREATE TABLE (\w+)"
missing escaped value, expected `b`, `e`, `f`, `n`, `r`, `\`, `"`, `x`, `u`, `U`
error: config file .../.specsync.toml exists but could not be loaded
```

`test_toml_full_config` wrote a regex in a TOML basic string. `\w` is not a valid escape. The old line scanner accepted it; `toml::from_str` (the #653 TOML fail-closed choke) rejects it; `load_config` then `process::exit(1)` kills the harness.

Fix the fixture with a literal string. Add a load_error test for the illegal escape so `load_config` is not used on it again.
