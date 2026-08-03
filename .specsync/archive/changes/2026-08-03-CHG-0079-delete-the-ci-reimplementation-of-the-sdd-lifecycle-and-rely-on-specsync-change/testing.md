---
change: CHG-0079-delete-the-ci-reimplementation-of-the-sdd-lifecycle-and-rely-on-specsync-change
artifact: testing
---

# Testing

## Requirement evidence

| Requirement | Evidence |
|-------------|----------|
| `REQ-github-008` | `cargo run -- change audit --strict` wired into the `spec-check` CI job as the single lifecycle authority; `cargo test` over `src/change.rs`; absence of any lifecycle reimplementation under `.github/`; `ci-gate` reduced to one aggregate context that no longer fails a green implementation to demand an archive-tip commit. |

## Retained validators

| Command | Result |
|---|---|
| `bash .github/scripts/test-classify-ci-paths.sh` | pass |
| `python3 -S .github/scripts/validate-workflow-runtime-pins.py` | pass — one `setup-bun@v2` with Bun 1.3.14 across 3 jobs |
| `python3 .github/scripts/test-validate-release-candidate.py` | pass — 50 tests |
| `cargo test` | see verification evidence |
| `cargo run -- check --strict --require-coverage 100 --force` | see verification evidence |
| `cargo run -- change audit --strict` | see verification evidence |

Every remaining workflow parses as YAML after the deletions.

## What is no longer tested, and why that is correct

The deleted harnesses tested the deleted code. `test-reuse-check-from-ancestors.py`,
`test-verify-trusted-policy-check.py`, and `test-lifecycle-workflows.sh` asserted properties of a
reimplementation that no longer exists. The properties worth keeping are asserted by
`specsync change audit --strict` and by the Rust suite over `src/change.rs`, both of which run on
every pull request.

## Regression risk accepted

CI no longer double-checks SpecSync with an independent implementation. A lifecycle defect in
`src/change.rs` is now caught by `cargo test` alone rather than by two disagreeing implementations.
This is the intended trade: the duplicate implementation blocked no bad merge and introduced two
defects of its own.
