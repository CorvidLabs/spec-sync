---
change: CHG-0016-preserve-free-text-arguments-in-generated-agent-commands
artifact: testing
---

# Testing

- Run `cargo test agents::` for template generation and install/idempotency coverage.
- Assert Claude, Cursor, and Gemini create-spec assets classify the complete input only after removing flags.
- Assert Gemini create-change contains `{{args}}` and no `$ARGUMENTS`.
- Assert all four skills and all three command assets quote `"<answer>"`.
- Reinstall all integrations twice and confirm the second install is a no-op.
- Run the full Fledge verification lane and strict SpecSync coverage.

## Requirement Evidence

- `REQ-agents-002`: `create_spec_commands_classify_the_complete_remaining_input`,
  `gemini_create_change_uses_native_args_and_quotes_answers`,
  `every_generated_lifecycle_surface_quotes_free_text_answers`, and
  `reinstall_keeps_all_generated_artifacts_byte_identical`.
