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
