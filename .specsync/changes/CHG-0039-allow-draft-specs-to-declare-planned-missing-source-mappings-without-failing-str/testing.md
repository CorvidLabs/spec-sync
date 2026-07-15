---
change: CHG-0039-allow-draft-specs-to-declare-planned-missing-source-mappings-without-failing-str
artifact: testing
---

# Testing

Focused integration fixtures will cover:

- a draft-only missing mapping that passes strict validation with a planned-mapping notice and exact zero impact on a non-vacuous coverage denominator;
- a mixed draft/active project where the draft path is planned but an active missing path still fails;
- changing draft to active, which restores the missing-file error;
- creating the planned file, which removes the notice and adds the real file to normal mapping and coverage;
- redundant dot-segment mappings, which are normalized consistently or rejected before planning;
- `require_draft_files = true` and legacy `requireDraftFiles`, which restore draft missing-file failures;
- duplicate ownership for existing files, including a changed spec that conflicts with an unchanged cached owner;
- rejection of unsafe draft paths and structured output notice separation;
- exact command, comment, and Markdown renderer signatures in canonical specs.

The final local gate is `fledge lanes run verify`, followed by `specsync check --strict --require-coverage 100 --force`, `git diff --check`, unfinished-artifact audit, and exact-head hosted matrices.

Focused regression commands:

- `cargo test require_draft_files`
- `cargo test --test integration draft_`
- `cargo test --test integration mixed_draft_and_active_missing_mappings_only_exempt_the_draft`
- `cargo test --test integration incremental_check_detects_duplicate_ownership_against_cached_specs`
- `cargo test --test integration draft_dot_segment_mapping_transitions_to_covered_file`

Canonical requirement evidence:

- `REQ-config-004`: configuration round-trip and legacy JSON tests plus the documented config table row.
- `REQ-commands-002`, `REQ-comment-002`, and `REQ-output-002`: structured and rendered notice regressions plus exact canonical signatures.
- `REQ-types-003` and `REQ-validator-007`: planned-mapping regressions validate notice separation, strict behavior, transitions, ownership, normalization, safety, and exact coverage.
