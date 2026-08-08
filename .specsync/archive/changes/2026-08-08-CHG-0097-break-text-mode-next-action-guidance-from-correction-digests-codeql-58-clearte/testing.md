---
change: CHG-0097-break-text-mode-next-action-guidance-from-correction-digests-codeql-58-clearte
artifact: testing
---

# Testing

## Unit

- `draft_text_surfaces_require_complete_artifacts_before_approval` in
  `src/commands/change.rs` — draft next-action recommends completing artifacts,
  never premature approve.
- Existing hash-TODO / HTML-TODO artifact incompleteness tests in
  `src/change.rs` still exercise `validate_artifacts` / body rules used by approve.

## Manual / local gate

```bash
cargo test --lib draft_text_surfaces_require_complete_artifacts_before_approval
cargo test --lib artifact_content_rejects_hash_todo
cargo fmt --check
cargo check
fledge lanes run pre-push
```

## Acceptance

- Text-mode status for a draft with incomplete artifacts suggests completing them
  without calling `effective_change_definition`.
- Approve still rejects incomplete effective artifacts after corrections.
- CodeQL #58 taint path (digests → text println) is removed; re-scan on merge
  should close or allow dismiss of residual FP only if JSON-only paths remain.
