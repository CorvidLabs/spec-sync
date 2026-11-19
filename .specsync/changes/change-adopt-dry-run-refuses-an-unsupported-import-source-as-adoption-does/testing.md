---
change: change-adopt-dry-run-refuses-an-unsupported-import-source-as-adoption-does
artifact: testing
---

# Testing

## Requirement evidence

| Contract | Evidence |
|---|---|
| `change adopt --dry-run` previews adoption; `--source` is openspec or speckit | `change::adopt_dry_run_refuses_an_unsupported_source_as_the_real_run_does` (red before the fix, green after) |
| A dry run writes nothing | the same test and the existing `adopt_dry_run_does_not_enable_policy` |

## How it runs

`change check` runs only the in-process spec-code sync. The tests run locally with
`cargo test --test integration adopt`, and in hosted CI's full suite at the archived head.
