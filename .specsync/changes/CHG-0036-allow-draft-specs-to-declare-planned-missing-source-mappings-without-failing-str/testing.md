---
change: CHG-0036-allow-draft-specs-to-declare-planned-missing-source-mappings-without-failing-str
artifact: testing
---

# Testing

Focused integration fixtures will cover:

- a draft-only missing mapping that passes strict validation with a planned-mapping notice and exact zero impact on a non-vacuous coverage denominator;
- a mixed draft/active project where the draft path is planned but an active missing path still fails;
- changing draft to active, which restores the missing-file error;
- creating the planned file, which removes the notice and adds the real file to normal mapping and coverage;
- `require_draft_files = true` and legacy `requireDraftFiles`, which restore draft missing-file failures;
- duplicate ownership for an existing file and rejection of unsafe draft paths;
- TOML serialization/default behavior and structured output notices.

The final local gate is `fledge lanes run verify`, followed by `specsync check --strict --require-coverage 100 --force`, `git diff --check`, placeholder audit, and exact-head hosted matrices.

Focused regression commands:

- `cargo test require_draft_files`
- `cargo test --test integration draft_`
- `cargo test --test integration mixed_draft_and_active_missing_mappings_only_exempt_the_draft`

Canonical requirement evidence:

- `REQ-config-004`: `config::tests::test_config_to_toml_roundtrips_require_draft_files` and `test_toml_and_legacy_json_read_require_draft_files`.
- `REQ-types-003`, `REQ-validator-007`, and `REQ-commands-002`: the five planned-mapping integration regressions validate notice separation, strict behavior, transitions, ownership, safety, and every check output format.

Review correction evidence:

- duplicate-ownership reporting records the existing files for each spec during the ownership pass, then performs direct lookups only for those files instead of scanning every project mapping for every spec;
- `cargo test --test integration draft_existing_files_keep_ownership_and_path_safety_validation` passes after the optimization.
