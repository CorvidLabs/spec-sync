---
id: CHG-0135-register-20-integration-tests-that-never-compiled-and-guard-against-orphaned-tes
state: implementing
type: bug_fix
base_commit: b5ca06da706eb61cf72cfdd2dce035f93b40bad6
---

# Register 20 integration tests that never compiled and guard against orphaned test files

## Intent

register 20 integration tests that never compiled and guard against orphaned test files

## Affected Canonical Specs

- None

## Acceptance Criteria

- tests/integration/regression_w1.rs is declared in tests/integration.rs and its 20 tests execute; cargo test reports 395 integration tests passing (374 baseline + 20 resurrected + 1 guard); every_integration_test_file_is_registered fails with rc=101 naming a planted orphan file and passes when removed; the two previously-failing tests pass on fixtures proven to discriminate, with --require-coverage 101 failing and 0 passing on the same tree; the marker count in regression_w1.rs is 20 before and after.

## No-spec Rationale

Registration and test-fixture work only; no module's public contract or spec text changes. tests/ is not owned by any spec module.
