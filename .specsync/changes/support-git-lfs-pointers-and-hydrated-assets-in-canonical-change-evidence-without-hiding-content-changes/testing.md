---
change: support-git-lfs-pointers-and-hydrated-assets-in-canonical-change-evidence-without-hiding-content-changes
artifact: testing
---

# Testing

Test pointer/hydrated parity, same-length edits, staged changes, deletion, empty files, malformed and extended pointers, other filters, and ignored/out-of-scope LFS in the change lifecycle. Use offline builds only. Current dependency resolution is blocked by missing cached serde-saphyr; full gate cannot be claimed.

## Observed validation, 2026-09-30

- cargo fmt --check passed.
- Four pure tests, extracted unchanged with their production helpers into a temporary sha2-only crate, passed using existing offline cached dependencies. The streaming fixture exceeds 256 MiB while enforcing 64 KiB maximum read buffers.
- Installed SpecSync strict validation passed: 62 specs, zero warnings, 107/107 files (100%). Changed module score: 100/100.
- Full workspace cargo metadata and mandatory CARGO_NET_OFFLINE=true fledge lanes run pre-push stop on missing cached serde-saphyr. The pre-push formatter step passed; cargo check did not compile the project. Git/lifecycle integration regressions have been added but not run.
- No PR is published while the mandatory pre-push gate is blocked. No implementation review, full-test pass, finalization or provenance approval is claimed.
- HI sentences were recorded only after Bruno explicitly confirmed them; hi is unavailable locally, so hi check was not run.

## Requirement evidence

| Requirement | Evidence |
| --- | --- |
| REQ-change-103 | src/change_tests.rs::lfs_payload_pointer_and_hydrated_bytes_are_equivalent; src/change_tests.rs::lfs_payload_refuses_malformed_and_extended_pointers; src/change_tests.rs::lfs_payload_streams_large_content_in_bounded_chunks; src/change_tests.rs::lfs_attribute_exception_is_specific_and_keeps_exact_output_checks; src/change_tests.rs::lfs_workspace_and_scoped_evidence_detect_edits_without_filters; src/change_tests.rs::lfs_unchanged_out_of_scope_assets_allow_change_check |

The first four tests ran in isolation; the last two are pending full-project compilation. This table maps assertions and does not claim the pending tests passed.
