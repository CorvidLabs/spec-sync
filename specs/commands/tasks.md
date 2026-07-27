---
spec: commands.spec.md
---

## Tasks

## Post-5.0 Test Debt

- [ ] Add focused inline tests in `src/commands/mod.rs` for `filter_specs`, `filter_by_status`, and `load_and_discover` (only `compute_exit_code` is currently unit-tested, and it lives in `src/main.rs`)

## Done

- [x] Initial spec creation with all required sections
- [x] Requirements and acceptance criteria documented
- [x] `compute_exit_code` exit-code matrix covered by unit tests in `src/main.rs` (warn/enforce-new/strict + require-coverage)
- [x] End-to-end enforcement and coverage flows covered in `tests/integration.rs`
- [x] Register verified SDD command dispatch under the shared command surface
- [x] Sanitize hostile repository, path, URL, and provider text in drift-creation terminal output
  and preserve the GitHub helper's title/body sanitization boundary
- [x] Preserve the public rendered `Vec<String>` validation API while retaining exact structured
  spec-path attribution privately, including legal paths containing `": "`
- [x] Enforce portable module names, Windows reserved basenames, trailing-space/dot rejection, and
  the generated spec filename byte limit with ASCII and multibyte boundary tests

## Gaps

- `filter_specs`/`filter_by_status` remain exercised indirectly; CHG-0063 adds focused inline
  coverage for exact drift-path attribution and public API compatibility.

## Review Status

Per-module role sign-offs were not collected. Release approval is governed by digest-bound change approvals and required CI; this note is informational and is not a release gate.
