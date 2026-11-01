---
change: keep-check-fix-export-rows-in-the-table-that-already-holds-documented-exports-768-case-1
artifact: testing
---

# Testing

Labels:

- A DISCRIMINATOR fails on the named base and passes with the fix.
- Bases: origin/main 709ab2bd, and #815's code (PR head 80658a63; the same src and test change
  is carried onto this branch first after approval). Labels are confirmed against builds of both before
  review and recorded here; they are not assumed.

## Requirement evidence

| Requirement | Evidence |
|-------------|----------|
| `REQ-cmd-check-017` | `fix_768_uses_one_contiguous_export_table` (layout a), `fix_768_keeps_rows_in_the_export_table_after_a_parameters_table` (layout c), `fix_768_ignores_a_name_headed_parameters_table_after_the_export_table` (layout d), all in `tests/integration/fix.rs` |
| `REQ-cmd-check-015` | the existing `fix_` suite in `tests/integration/fix.rs` and `tests/integration.rs`, unchanged and passing |

## Integration tests (`tests/integration/fix.rs`)

All three reuse #815's `assert_fix_768_table`, generalized to take the existing export row and
the expected cell count. It asserts that the `b` row has that width, directly follows the
existing row, that the file equals the input plus that one row, that `check --strict --force`
passes, and that a second `check --fix` is a no-op.

- DISCRIMINATOR on origin/main `fix_768_uses_one_contiguous_export_table` (layout a, from #815):
  headed, bold-labelled and flat sections; with and without a leading parameters table; `Name`
  and `Function` headers.
- DISCRIMINATOR on #815 `fix_768_keeps_rows_in_the_export_table_after_a_parameters_table`
  (layout c): `| Parameter | Description |` then `| Function | Description |` holding `a`. Passes
  on origin/main; that is the point: it pins main's correct behavior that #815 broke.
- DISCRIMINATOR on both `fix_768_ignores_a_name_headed_parameters_table_after_the_export_table`
  (layout d): `| Function | Description |` holding `a`, prose, then
  `| Name | Type | Description |`.

Header-rule and fallback coverage (only reached when no table holds a documented row) is an
open question in `tasks.md`; the draft proposes one extra test for each.

## Regression lanes

- `cargo test --test integration fix_` (every existing `--fix` test passes unchanged).
- `cargo test`, `cargo clippy -- -D warnings`, `cargo fmt --check`.
- `specsync check --strict --require-coverage 100 --force` and `fledge lanes run pre-push` before
  any push.
