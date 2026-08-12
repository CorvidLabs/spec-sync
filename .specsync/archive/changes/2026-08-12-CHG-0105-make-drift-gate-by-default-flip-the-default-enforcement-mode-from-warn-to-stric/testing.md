---
change: CHG-0105-make-drift-gate-by-default-flip-the-default-enforcement-mode-from-warn-to-stric
artifact: testing
---

# Testing

## Approach

The blast radius was established by applying the one-line change and running the whole suite
*before* the change workspace existed, rather than discovering it at successive gates. Four
call sites in two test files needed pinning; nothing else moved.

Three tests were **adapted, not deleted** — the opposite of the Step 1 decision, and
deliberately so. They do not assert the changed behaviour; their fixtures are intentionally
incomplete (so `--fix` has something to repair) or intentionally duplicated (so draft
skipping can be observed), and they assert on reported output. Pinning them to
`--enforcement warn` preserves their intent and states in a comment that the fixture carries
real findings.

## Commands

- `cargo test` — 2197 unit + 331 integration, 0 failed
- `cargo clippy -- -D warnings` — clean
- `cargo fmt --check` — clean

## Requirement evidence

| Requirement | Evidence |
|-------------|----------|
| REQ-types-004 | `cargo test`; a tree with a validation error now exits 1 from a bare `specsync check`, and `--enforcement warn` restores exit 0 |
