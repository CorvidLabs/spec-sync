# Tasks

- [x] Verify each of the four findings against the tree rather than accepting the report
- [x] `agents.rs`: manifest structs tolerate unknown fields, required fields still required
- [x] `change.rs`: correct the comment that misclassified `agents.rs`
- [x] `change.rs`: `SddPolicy` container-level `#[serde(default)]`
- [x] `change_tests.rs`: cache test rewritten against `hashes`, asserts the unknown field
- [x] `change_tests.rs`: baseline case swapped for `FinalizationRecord`
- [x] `change_tests.rs`: canonical-bytes limit pinned
- [x] `change_tests.rs`: policy test with fail-closed assertions
- [x] `agents.rs`: manifest test with its refusal control
- [x] Discrimination measured in a scratch copy
- [x] Requirements amended and added
- [x] Full suite, clippy, fmt
- [x] Lifecycle: check --commit, review, ship, archive
