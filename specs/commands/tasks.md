---
spec: commands.spec.md
---

## Tasks

- [ ] Add focused inline tests in `src/commands/mod.rs` for `filter_specs`, `filter_by_status`, and `load_and_discover` (only `compute_exit_code` is currently unit-tested, and it lives in `src/main.rs`)

## Done

- [x] Initial spec creation with all required sections
- [x] Requirements and acceptance criteria documented
- [x] `compute_exit_code` exit-code matrix covered by unit tests in `src/main.rs` (warn/enforce-new/strict + require-coverage)
- [x] End-to-end enforcement and coverage flows covered in `tests/integration.rs`
- [x] Register verified SDD command dispatch under the shared command surface

## Gaps

- `src/commands/mod.rs` has no `#[cfg(test)]` module; `filter_specs`/`filter_by_status`/`run_validation` are exercised only indirectly via integration tests

## Review Sign-offs

- **Product**: pending
- **QA**: pending
- **Design**: n/a
- **Dev**: pending
