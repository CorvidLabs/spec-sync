//! Unit tests for [`crate::change`].
//!
//! Split out of `change.rs` by #589: that file was 29,983 lines — 23% of the
//! codebase — and its size is not cosmetic. The defect this release has spent
//! its correctness campaign on is a fix landing at the site named in the bug
//! report while a parallel implementation survives, which has now happened
//! seven times. You cannot sweep for a sibling in a file you cannot hold in
//! your head.
//!
//! This is a PURE MOVE. No test was added, removed, renamed or edited; the
//! `#[test]` count is identical before and after. `#[path]` keeps the module
//! inline for the compiler, so `use super::*` still reaches every private item
//! exactly as before.
//!
//! The 24 `#[cfg(test)]` helpers and fault-injection hooks stay in `change.rs`:
//! production code paths reference them, so they are not test code that merely
//! lives near production code.

include!("change_tests/substantive_plan.rs");
include!("change_tests/archive_post_move_failure_restores_exact_source_bytes_without_residue.rs");
include!("change_tests/approve_rejects_a_declared_path_owned_by_an_undeclared_module.rs");
include!("change_tests/semantic_application_resolves_registry_backed_canonical_paths.rs");
include!("change_tests/canonical_resolution_tolerates_inert_legacy_registry_stub.rs");
include!("change_tests/verification_ignored_paths.rs");
include!("change_tests/approved_delta_requirements.rs");

include!("change_tests/recorded_review_recovery.rs");

include!("change_tests/historical_reconciliation.rs");

include!("change_tests/fast_archived_reads.rs");
include!("change_tests/upstream_archive.rs");

include!("change_tests/fast_finalize.rs");
include!("change_tests/easier_supersede.rs");
include!("change_tests/approve_ownership_names_the_spec.rs");
include!("change_tests/stale_modified_delta.rs");
include!("change_tests/send_back_for_rescope.rs");
include!("change_tests/relayed_decision.rs");
include!("change_tests/open_task_gate.rs");
include!("change_tests/approve_refusal_hints.rs");
include!("change_tests/doctor_collision.rs");

include!("change_tests/recovery_retirement.rs");
include!("change_tests/transaction_journal_error.rs");
include!("change_tests/titled_requirement_header_delta.rs");
include!("change_tests/bounded_failure_output.rs");
include!("change_tests/inert_ignored_paths.rs");
include!("change_tests/commit_identity_guards.rs");
include!("change_tests/unknown_members.rs");
