---
spec: util.spec.md
---

## Tasks

- [x] Document shared utility helpers in a dedicated spec
- [x] Project file functions: read, write, create and remove beneath the project root without leaving it through a link (REQ-util-002). Evidence: the five `project_file_functions_*` tests.

## Gaps

- `safe_regex` currently tests invalid syntax and valid patterns; oversized compiled-regex behavior depends on regex crate internals and is not covered directly.

## Post-5.0 Test Debt

- [ ] Add boundary tests for very large regex patterns if the regex crate exposes stable size-limit errors

## Review Status

Per-module role sign-offs were not collected. Release approval is governed by digest-bound change approvals and required CI; this note is informational and is not a release gate.
