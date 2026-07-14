---
change: CHG-0033-close-final-5-0-2-lifecycle-review-and-intent-preservation-gaps
artifact: testing
---

# Testing

## Focused regression coverage

- `REQ-change-030`:
- Reject recursive SpecSync verification selected by both `--manifest-path <path>` and `--manifest-path=<path>` before attempt/state mutation.
- Continue allowing a non-SpecSync manifest selected through `--manifest-path`.
- Cover each standard canonical companion (`requirements.md`, `tasks.md`, `context.md`, `testing.md`, and `design.md`).
- Reject implicit coverage for unrelated siblings and for a different module sharing the directory.
- `REQ-change-031`:
- Preserve commas and embedded newlines in a scalar acceptance criterion.
- Parse an explicit JSON string array as multiple criteria.
- Preserve comma/newline parsing for affected specs and paths.
- Verify persisted state and rendered `change.md` retain exact criterion text.

## Full verification

- Run the focused Rust tests for lifecycle command classification, coverage, and interview parsing.
- Run the repository test, formatting, lint, strict SpecSync, and Trust verification lanes.
- Record and verify Attest provenance after the verification lane passes.
