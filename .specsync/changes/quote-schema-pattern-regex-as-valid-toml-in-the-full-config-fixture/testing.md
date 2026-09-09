---
change: quote-schema-pattern-regex-as-valid-toml-in-the-full-config-fixture
artifact: testing
---

# Testing

Fail: `cargo test --bin specsync -- --test-threads=8 --nocapture` exits 1 at `test_toml_full_config` via `refuse_unloadable_config`.

Pass:

```
cargo test --bin specsync -- test_toml_full_config test_toml_invalid_schema_pattern_escape_sets_load_error
cargo test --bin specsync -- --test-threads=8
```

The full bin suite must print `test result: ok` rather than "exited abnormally".
