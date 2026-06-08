---
spec: ignore.spec.md
---

## Done

- [x] `WarningCategory` enum with `from_str` (aliases, case/`_`-`-` insensitive) and `classify` (text → category)
- [x] `IgnoreRules::load` for `.specsyncignore` (global + per-spec rules, comment handling, missing-file safe)
- [x] `IgnoreRules::parse_inline` for `<!-- specsync-ignore: ... -->` directives
- [x] `is_suppressed` covering global, inline, and per-spec prefix scopes
- [x] Inline unit tests for classify ordering, aliases, parse_inline, all suppression scopes, and load (present/absent)
- [x] Initial spec creation with all required sections
- [x] Requirements and acceptance criteria documented

## Open

- [ ] No end-to-end integration test that asserts a `.specsyncignore` rule actually suppresses a warning in `specsync check` output
- [ ] Per-spec patterns are prefix-only; glob support is not implemented

## Review Sign-offs

- **Product**: pending
- **QA**: pending
- **Design**: n/a
- **Dev**: pending
