---
change: decide-canonical-materialization-from-the-artefacts-instead-of-the-canonical-applied-flag-so-a-delta-corrected-after
artifact: testing
---

# Testing

Every assertion below was failed against a binary built from a **separate clean checkout of
unfixed `main`** at `7df4077`, with only these tests added to it. Nothing was reverted in place.

## Evidence for REQ-change-092

| Test | Label | Covers |
|------|-------|--------|
| `a_delta_corrected_after_materialization_reaches_the_canonical_spec_on_the_next_check` | DISCRIMINATOR | The filed defect: the corrected wording reaches the spec and the superseded wording does not survive |
| `a_materialized_spec_missing_its_version_bump_and_change_log_row_gets_both_back` | DISCRIMINATOR | The widening: the two outputs no delta digest can derive |
| `re_approving_a_byte_identical_delta_leaves_the_canonical_spec_byte_for_byte_alone` | **CONTROL** | "Always re-materialize" is not the fix; the spec, the version and the row are all left alone |
| `a_corrected_delta_re_materializes_over_a_block_its_own_earlier_run_removed` | DISCRIMINATOR | Re-materialization does not refuse the removal it performed itself |
| `the_refusal_for_a_changed_delta_names_the_second_step_that_finishes_the_job` | DISCRIMINATOR | The diagnostic names `check` as well as `approve` |

## Verbatim control failures (unfixed `main`)

```
thread '...a_delta_corrected_after_materialization_reaches_the_canonical_spec_on_the_next_check'
panicked at src/change_tests.rs:14034:5:
the corrected wording must reach the canonical spec: ---
module: auth
version: 1.0.1
...
## Purpose

Auth tracks credentials. Reviewed and approved wording.
```

```
thread '...a_materialized_spec_missing_its_version_bump_and_change_log_row_gets_both_back'
panicked at src/change_tests.rs:14090:5:
a spec carrying this change's contract text must carry its version bump: ---
module: auth
version: 1.0.0
```

```
thread '...a_corrected_delta_re_materializes_over_a_block_its_own_earlier_run_removed'
panicked at src/change_tests.rs:14197:5:
the corrected section must reach the canonical spec: ---
module: auth
version: 1.0.1
...
## Purpose

Auth.
```

```
thread '...the_refusal_for_a_changed_delta_names_the_second_step_that_finishes_the_job'
panicked at src/change_tests.rs:14230:5:
the remedy must name the step that puts the approved wording in the canonical spec; naming only
`approve` walked the author into the silent skip: semantic delta for `auth` changed after
approval; the approved wording is what rewrites the canonical spec, so re-run `specsync change
approve add-passkeys` to approve the current delta bodies (or restore them)
```

The CONTROL passes on that same unfixed binary, as it must:

```
test change::tests::re_approving_a_byte_identical_delta_leaves_the_canonical_spec_by
te_for_byte_alone ... ok
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 2423 filtered out
```

## Measurements, not arguments

- 208 `state.json` (193 with `canonical_applied: true`) and 208 `approvals.json` under `.specsync/`
  all deserialize and round-trip under the changed binary. No field was added to `ChangeRecord`,
  so no persisted shape moved and no definition digest moved.
- 446 of 454 archived (change, module) pairs carry a Change Log row naming the change in the
  module's current spec, which is what makes the row usable as per-module materialization
  evidence. The eight exceptions are all pre-6.0 archived changes that `check` never revisits.

## Gates

- `cargo fmt --check` clean.
- `cargo clippy -- -D warnings` clean.
- Full `cargo test` green (`change::tests::` 419 passed, 0 failed).
