---
change: CHG-0032-address-all-actionable-review-findings-on-pr-370-with-regression-coverage
artifact: testing
---

# Testing

## Requirement Evidence

- `REQ-change-029`: `valid_later_sequence_claim_preserves_historical_acceptance_input`, `later_collision_acknowledgements_do_not_stale_earlier_sequence_evidence`, and `current_sequence_owner_binds_exact_ledger_content` prove historical stability and current-owner exactness.
- `REQ-change-030`: `native_cargo_check_argument_is_not_misclassified_as_specsync`, `semantic_application_resolves_registry_backed_canonical_paths`, and the protected-path assertions prove recursive command classification, exact canonical-file coverage, both registry authorities, and robust Cargo package parsing.

## Verification Boundary

- Add focused unit regressions for all five review findings.
- Run `cargo fmt --check`, `cargo clippy -- -D warnings`, and `cargo test`.
- Run strict candidate SpecSync validation at 100% file and LOC coverage.
- Require the exact pushed head to pass the full hosted CI, Trust, CodeQL, packaged-action, and operating-system matrices.
