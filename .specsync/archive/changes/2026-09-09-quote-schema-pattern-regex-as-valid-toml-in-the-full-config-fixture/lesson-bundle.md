# Lesson bundle — quote-schema-pattern-regex-as-valid-toml-in-the-full-config-fixture

Material for folding this change's lessons into the affected specs' `context.md`.
Synthesise from what actually happened below; do not restate the change description.

## What this change was

- **Title**: Quote schema_pattern regex as valid TOML in the full-config fixture
- **Kind**: BugFix
- **Specs**: config
- **Paths**: src/config.rs
- **Acceptance**: cargo test --bin specsync completes without the harness exiting abnormally; test_toml_full_config loads and asserts schema_pattern is CREATE TABLE (\w+); a double-quoted illegal TOML escape in schema_pattern sets load_error via load_config_allowing_unloadable

## Evidence

- Verification commit: `b23a4b20862de6fd0af10e9f91ef3e44a8408827`
- Base commit: `a9169aacb0d8807c85dd5e752ebe32453d9c9b8b`
- Verified by: `specsync check --spec config`

## From the change's context.md

# Context

`cargo test --bin specsync` dies after the last `config::` companions test with "test exited abnormally" and no FAILED row. `--nocapture` shows:

```
schema_pattern = "CREATE TABLE (\w+)"
missing escaped value, expected `b`, `e`, `f`, `n`, `r`, `\`, `"`, `x`, `u`, `U`
error: config file .../.specsync.toml exists but could not be loaded
```

`test_toml_full_config` wrote a regex in a TOML basic string. `\w` is not a valid escape. The old line scanner accepted it; `toml::from_str` (the #653 TOML fail-closed choke) rejects it; `load_config` then `process::exit(1)` kills the harness.

Fix the fixture with a literal string. Add a load_error test for the illegal escape so `load_config` is not used on it again.

## From the change's testing.md

# Testing

Fail: `cargo test --bin specsync -- --test-threads=8 --nocapture` exits 1 at `test_toml_full_config` via `refuse_unloadable_config`.

Pass:

```
cargo test --bin specsync -- test_toml_full_config test_toml_invalid_schema_pattern_escape_sets_load_error
cargo test --bin specsync -- --test-threads=8
```

The full bin suite must print `test result: ok` rather than "exited abnormally".

## Where these lessons go

- `specs/config/context.md`
