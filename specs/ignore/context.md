---
spec: ignore.spec.md
---

## Key Decisions

- **Three suppression scopes**: global (bare category in `.specsyncignore`), per-spec (`category:path-prefix`), and inline (`<!-- specsync-ignore: ... -->` in the spec body). `is_suppressed` checks all three.
- **Classify by message text**: warnings are categorized by matching substrings/prefixes of their text (`WarningCategory::classify`), so the suppression layer stays decoupled from where warnings are emitted. Order matters — `schema-type-mismatch` is checked before the generic `schema-column`.
- **Forgiving parsing**: `from_str` lowercases input, treats `_`/`-` as equivalent, and accepts short aliases, so users don't have to memorize exact variant names.
- **Missing file is not an error**: `load` returns empty rules when `.specsyncignore` is absent, so the feature is purely opt-in.
- **Prefix matching, not globs**: per-spec rules match when the spec's relative path `starts_with` the configured pattern.

## Files to Read First

- `src/ignore.rs` — `WarningCategory`, `IgnoreRules`, and all inline tests
- `src/validator.rs` / `src/commands/check.rs` — call `is_suppressed` to filter warnings before reporting

## Current Status

Fully implemented and stable, with inline unit tests covering `classify`, `from_str` aliases, `parse_inline`, all three `is_suppressed` scopes, and `load` (present + absent file).

## Notes

- This is a library module consumed by the validation/check path — it does no config loading or process exit of its own.
- Each new suppressible warning needs a `WarningCategory` variant plus arms in both `from_str` and `classify`.
