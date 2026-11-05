# Lesson bundle: make-the-unowned-path-ownership-hint-deterministic-and-fall-back-at-depth-zero

Material for folding this change's lessons into each affected module's lesson file.
Synthesise from what actually happened below; do not restate the change description.

## What this change was

- **Title**: Make the unowned-path ownership hint deterministic and fall back at depth zero
- **Kind**: BugFix
- **Specs**: change
- **Paths**: src/change/sdd_version.rs, src/change_tests/approve_ownership_names_the_spec.rs
- **Acceptance**: `ownership_hint` picks the first declared spec even at depth 0, so `lib/new.ts` can be pointed at `specs/billing/billing.spec.md`. Guard on `depth > 0` and fall back to the generic message.
- **Acceptance**: The first-pass walk returns the first undeclared spec that lists the path, in `read_dir` order. That order isn't deterministic across platforms. Collect every match and sort.
- **Acceptance**: Missing tests: the depth-0 fallback, `src/foo` vs `src/foobar`, a glob entry, and the `_ =>` arm.

## Evidence

- Verification commit: `3cf5ab2e34e03c5834a68916bf61817d2ddbac52`
- Base commit: `5e137ece36437d8eaa5b54f481457d8f7444ee49`
- Verified by: `specsync check --spec change`

## From the change's context.md

# Context

Leif approved #841 on GitHub (review 5771062363, 2026-11-04 08:33 MST) with optional nits, and at 09:25 MST said: "Yes, merge #841 and open the nits follow-up PR." #841 merged as f65dbed. The nits are captured verbatim in hi as CHANGE-10.a, CHANGE-10.b and CHANGE-10.c.

Ownership does not change (REQ-change-053). `ownership_hint` only words the refusal after it has been decided.

Beyond the nits, this branch also carries two refinements from the SpecSync bot's review of #841 that Leif has not yet confirmed and should accept or reject in review: an undeclared module whose `[modules."<name>"] owns` covers the path is named, and equally close declared specs are all named instead of the first in `affected_specs` order.

## From the change's testing.md

# Testing

Run with `cargo test --bin specsync approve_ownership_refusal`.

- `approve_ownership_refusal_at_depth_zero_falls_back_to_the_generic_message` (CHANGE-10.a) fails on 02818ce and on main f65dbed, and passes with the fix.
- `approve_ownership_refusal_names_every_undeclared_spec_that_lists_the_path_in_order` (CHANGE-10.b) fails on 0d04443/f65dbed and passes with the fix.
- `approve_ownership_refusal_matches_whole_directory_components` (`src/foo` vs `src/foobar`) fails on 02818ce and passes on main and with the fix.
- `approve_ownership_refusal_scores_a_glob_entry_by_its_directory` is coverage: it passes on main too, as Leif's review says globs score correctly.
- `approve_ownership_refusal_explains_a_listing_spec_without_a_module` and `approve_ownership_refusal_explains_a_declared_module_listing_outside_its_canonical_spec` cover the two cases of the `_ =>` arm and fail on main f65dbed.
- `approve_rejects_a_declared_path_owned_by_an_undeclared_module` still passes, so the same paths are refused.

## Requirement evidence

| Requirement | Evidence |
| --- | --- |
| REQ-change-120 | `approve_ownership_refusal_at_depth_zero_falls_back_to_the_generic_message`, `approve_ownership_refusal_names_every_undeclared_spec_that_lists_the_path_in_order`, `approve_ownership_refusal_matches_whole_directory_components`, `approve_ownership_refusal_scores_a_glob_entry_by_its_directory`, `approve_ownership_refusal_explains_a_listing_spec_without_a_module`, `approve_ownership_refusal_explains_a_declared_module_listing_outside_its_canonical_spec`, `approve_ownership_refusal_names_the_declared_spec_to_edit`, `approve_ownership_refusal_names_an_undeclared_spec_that_lists_the_path`, `approve_rejects_a_declared_path_owned_by_an_undeclared_module` |

## Where these lessons go

- `specs/change/change.3md`
