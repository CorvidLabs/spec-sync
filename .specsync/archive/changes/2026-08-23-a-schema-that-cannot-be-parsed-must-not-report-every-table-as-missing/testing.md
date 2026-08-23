---
change: a-schema-that-cannot-be-parsed-must-not-report-every-table-as-missing
artifact: testing
---

# Testing

## Reproduction, before and after

    before:  ✗ DB table not found in schema: events
             ✗ DB table not found in schema: visitors
             ✗ ALTER TABLE ADD duplicates existing column `events.url`
             0 passed, 1 failed

    after:   ✓ All DB tables exist in schema
             1 passed, 0 failed

## Discriminator vs control — stated precisely, because they are not the same

| test | baseline | fixed | what it proves |
|---|---|---|---|
| `an_agreeing_add_column_redeclaration_is_a_no_op` | FAILS | passes | true discriminator: real behaviour change |
| `a_contradicting_add_column_redeclaration_still_fails` | fails (wording only) | passes | message discriminator, NOT behavioural |
| `the_duplicate_column_check_still_rejects_a_type_conflict` | PASSES | PASSES | true vacuity control |

The second test was initially mistaken for a control. On the baseline the contradicting case DOES
error — with the old wording "duplicates existing column" — so it fails only on the `contains(
"redeclares")` assertion. Behaviour is identical on both binaries there.

The third test was added once that was noticed. It asserts behaviour only, no message text, so it
passes on both. A fix that simply DELETED the duplicate-column check would pass the discriminator
and fail this one. That is the property worth pinning.

## Live controls on the reproduction

    CONTROL 1  a genuinely missing table  -> ✗ DB table not found in schema: ghosts       (still fires)
    CONTROL 2  TEXT then INTEGER          -> ✗ redeclares ... (`TEXT` then `INTEGER`)     (still fatal)
    CONTROL 3  ALTER on a missing table   -> ✗ references missing canonical table ...     (ONLY error;
               previously events and visitors were also reported absent)

Control 3 is the cascade fix demonstrated end to end: one real error instead of one real error
buried under two false ones.

## Suite

`cargo test`: 2349 unit + 405 integration, 0 failed. `cargo fmt --check` clean.

## Not covered

Whether an agreeing redeclaration deserves a project-scope warning. Rejected for now: warnings gate
under `strict`, which is the mode this change exists to unblock. Recorded on #672 rather than
decided silently.
