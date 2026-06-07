---
spec: util.spec.md
---

## Tasks

- [x] Document shared utility helpers in a dedicated spec
- [ ] Add boundary tests for very large regex patterns if the regex crate exposes stable size-limit errors

## Gaps

- `safe_regex` currently tests invalid syntax and valid patterns; oversized compiled-regex behavior depends on regex crate internals and is not covered directly.

## Review Sign-offs

- **Product**: pending
- **QA**: pending
- **Design**: n/a
- **Dev**: pending
