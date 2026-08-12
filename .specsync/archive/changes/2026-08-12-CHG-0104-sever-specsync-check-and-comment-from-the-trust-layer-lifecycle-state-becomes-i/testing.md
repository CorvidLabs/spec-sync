---
change: CHG-0104-sever-specsync-check-and-comment-from-the-trust-layer-lifecycle-state-becomes-i
artifact: testing
---

# Testing

## Approach

Unit coverage via the `cmd_check` component command. Behavioural confirmation via
the sandbox drills, which are the judge for this class of change: the Rust suite is
single-process and single-root and cannot observe the multi-repo conditions that
made the trust gate fail in the first place.

The decisive external check is sandbox drill 038, which pins the drift invariant
with SDD disabled. It must stay green: if severing the gate breaks it, the
reduction broke the product.

## Commands

- `cargo test commands::check::tests::`
- `SPECSYNC=<binary> drills/038-check-drift-invariant.sh`
- `SPECSYNC=<binary> drills/028-ship-lifecycle-dogfood.sh`

## Removed tests

Three integration tests asserted the behavior being removed and are deleted, not adapted:
`sdd_failure_json_preserves_check_schema` (`tests/integration/check.rs`),
`comment_reports_sdd_only_failures` and `comment_reports_sdd_failures_when_no_specs_exist`
(`tests/integration/comment.rs`). Full suite green afterwards: 2197 unit, 331 integration.

## Requirement evidence

| Requirement | Evidence |
|-------------|----------|
| REQ-cmd-check-004 | `cargo test commands::check::tests::`; a repo with an uncovered meaningful path now exits 0 from `specsync check` while still reporting the active-change count |
| REQ-change-058 | `cargo test` (full suite: `change` is in the hardcoded strict-module list and `src/change.rs` is a strict path); `cargo clippy -- -D warnings` reports no dead code |
| REQ-cmd-comment-004 | `cargo test commands::check::tests::`; `specsync comment` output contains no `.specsync/sdd.json:` prefixed entries |
