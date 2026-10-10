---
change: split-change-command-dispatch-rendering-shipping-and-lifecycle-helpers-into-private-modules
artifact: testing
---

# Testing

## Verification plan

Record pristine `cargo test --bin specsync commands::change -- --list` and focused test output, then compare the relocated test-name inventory and outcomes. Compare complete function bodies after ignoring only module indentation and authorized visibility prefixes. Run `cargo fmt --check`, `cargo check`, focused command tests, clippy and strict source coverage. Review snapshot/read-scope, trust/readiness, NUL-aware staging and gate-before-push seams independently. Run actual `specsync change check <id> --commit` without inventing review or acceptance.

## Environment limits

Use the installed shared Rust toolchain and target; install nothing. Fledge, Attest and Ruby are unavailable in the current environment. Existing full-main baseline failures are due to sandbox Git-fixture ancestry; any observed differences must be reported precisely. A focused test or scoped spec check is not a full trust pass.

## Recorded results before lifecycle materialization

- Pristine and split focused command runs: 29 tests, 22 passed and the same seven Git repository-detection failures; leaf-name inventory unchanged.
- Full unit comparison: the same 2530 test names, 2338 passed and 192 failed, with identical failure sets after normalizing only the three relocated command-test namespaces.
- Full integration comparison: the same 438 test names, 425 passed and 13 failed, with identical failure sets.
- `cargo check --tests` passed, retaining the pre-existing `src/change_tests.rs:3813` unused-mut warning.
- `cargo clippy --bin specsync -- -D warnings` and `cargo fmt --check` passed.
- `cargo clippy --all-targets -- -D warnings` reports 31 diagnostics in untouched code: `src/change.rs`, `src/change_tests.rs`, `src/commands/init.rs`, `src/commands/scaffold.rs`, `src/hash_cache.rs`, `src/validator.rs` and `tests/integration/change.rs`. The known unused-mut/collapsible-if diagnostics are included; no diagnostic targets the extracted command. No suppression or unrelated fix was added.
- Strict whole-project synchronization and 100% file/LOC coverage passed before delta materialization: 62 specs, zero warnings or failures, 115/115 covered files.
- Independent AST/visibility/import review found no blocking change; all original bodies, literals, docs, attributes and tests remain. This is agent technical review, not human implementation review.
- `fledge lanes run verify` cannot execute: `fledge` is not installed (exit 127). Attest and Ruby are also unavailable. These limitations do not constitute a full trust pass.

`cargo build --release` passed. `bash scripts/pre-push-gate.sh` passed all three steps using the freshly built release binary; the script itself is unchanged (direct execution was unavailable because its checked-out mode lacks execute permission). `fledge trust verify` could not execute because Fledge is absent (exit 127). The prerequisite task checklist covers implementation and validation; actual scoped materialization/check/commit follows it and stores its own evidence. Human scoped implementation review, publication and archival are still pending and are not implied by any local result.
