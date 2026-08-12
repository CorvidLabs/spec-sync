---
change: CHG-0106-make-verification-currency-a-content-question-delete-the-git-ancestry-walk-the
artifact: testing
---

# Testing

## Approach

The Rust suite cannot judge this change alone: it is single-process and single-root, and the
behaviour being removed concerns squash merges and commit topology. The sandbox drills are the
judge and were run against a release binary built from this tree.

## Results

`cargo test` — 2189 unit + 331 integration, 0 failed. `cargo clippy -- -D warnings` and
`cargo fmt --check` clean.

| Drill | Result |
|-------|--------|
| 038 drift invariant | PASS 10/10 |
| 028 ship lifecycle happy path | PASS 15/15 |
| 036 concurrent check serialization | PASS 8/8 |
| 032 same-PR archive path coverage | PASS 7/7 |
| 026 multi-clone approve | PASS |
| 008 squash-archive regression | behaviour changed as predicted |

008 previously asserted that a squash merge before finalize strands the change on stale
verification. It no longer does; it stops at scoped-review staleness instead. That is the
verification half of squash orphaning dissolving, with the review half remaining for the next
step. The drill is updated alongside the candidate-SHA bump, together with drill 037's
`check --strict` assertion, rather than being edited mid-reduction against an older candidate.

## Removed tests

Eight unit tests in `src/change.rs` asserted the deleted behaviour and were deleted with it,
not adapted: the persistence allowlist, verification-commit canonicality and ancestry, nested
project prefix stripping, and the change-then-revert family. `#[test]` count 308 to 300,
verified by count rather than by trusting the removal output.

## Requirement evidence

| Requirement | Evidence |
|-------------|----------|
| REQ-change-013 | `cargo test`; freshness decided by the three content equalities with no git invocation in that path |
| REQ-change-016 | drill 008 — a squash-merged change is no longer stranded on verification staleness |
