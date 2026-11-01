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

- DISCRIMINATOR on both `fix_768_uses_one_contiguous_export_table` (layout a, from #815):
  headed, bold-labelled and flat sections; with and without a leading parameters table, each
  with `Name` and `Function` headers. #815's version tried `Function` only without the leading
  parameters table; review asked for the combined case, which #815's code fails (it takes the
  width from the parameters table).
- DISCRIMINATOR on #815 `fix_768_keeps_rows_in_the_export_table_after_a_parameters_table`
  (layout c): `| Parameter | Description |` then `| Function | Description |` holding `a`. Passes
  on origin/main; that is the point: it pins main's correct behavior that #815 broke.
- DISCRIMINATOR on both `fix_768_ignores_a_name_headed_parameters_table_after_the_export_table`
  (layout d): `| Function | Description |` holding `a`, prose, then
  `| Name | Type | Description |`.

Header-rule and fallback coverage (only reached when no table holds a documented row), added on
approval:

- DISCRIMINATOR on both `fix_768_header_rule_picks_the_export_table_when_none_holds_a_row`:
  `| Name | Type | Description |` parameters table, an empty `| Function | Description |`
  table, prose, then `| Field | Description |`. Rows for `a` and `b` (2 cells) follow the Function
  table's separator.
- DISCRIMINATOR on both `fix_768_falls_back_to_the_last_table_with_its_own_width`:
  `| Parameter | Type | Description |`, prose, then an empty `| Item | Description |` table. Rows
  (2 cells) follow the last table's separator.

Each test runs under a `### Exported Functions` heading, a `**Functions**` label and no label.

## Regression lanes

- `cargo test --test integration fix_` (every existing `--fix` test passes unchanged).
- `cargo test`, `cargo clippy -- -D warnings`, `cargo fmt --check`.
- `specsync check --strict --require-coverage 100 --force` and `fledge lanes run pre-push` before
  any push.

## Results

Labels confirmed by running `cargo test --locked --test integration fix_768` with the current
tests against each base's `src/commands/check.rs`:

| Test | origin/main 709ab2bd | #815 (carried, 40c6a4a0) | fix (2e36b08b) |
|------|----------------------|--------------------------|----------------|
| `fix_768_uses_one_contiguous_export_table` (a) | FAIL: rows directly follow the target table's last row | FAIL: must use the target table's width (`Function` header after a parameters table) | pass |
| `fix_768_keeps_rows_in_the_export_table_after_a_parameters_table` (c) | pass | FAIL: rows directly follow the target table's last row | pass |
| `fix_768_ignores_a_name_headed_parameters_table_after_the_export_table` (d) | FAIL: rows directly follow the target table's last row | FAIL: must use the target table's width | pass |
| `fix_768_header_rule_picks_the_export_table_when_none_holds_a_row` | FAIL: must use the target table's width | FAIL: must use the target table's width | pass |
| `fix_768_falls_back_to_the_last_table_with_its_own_width` | FAIL: must use the target table's width | FAIL: must use the target table's width | pass |

Regression lanes on the fix:

- `cargo test --locked --test integration fix`: 32 passed, 0 failed.
- `cargo fmt --check`: clean. `cargo clippy -- -D warnings`: clean.
