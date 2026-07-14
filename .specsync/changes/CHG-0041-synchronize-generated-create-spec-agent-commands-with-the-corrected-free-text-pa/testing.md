---
change: CHG-0041-synchronize-generated-create-spec-agent-commands-with-the-corrected-free-text-pa
artifact: testing
---

# Testing

This plan provides implementation evidence for `REQ-agents-003`.

## Focused regression

- Extend `create_spec_commands_classify_the_complete_remaining_input` to assert that flag removal
  appears before classification and all four before/after flag examples are rendered for Claude,
  Cursor, and Gemini.
- Assert generated content contains `Never use only the first word as the module name` and does not
  contain `first whitespace-separated token`.
- Generate all three commands in a temporary project and byte-compare them with
  `.claude/commands/specsync/create-spec.md`, `.cursor/commands/specsync-create-spec.md`, and
  `.gemini/commands/specsync/create-spec.toml` from the repository.
- Run a second install and assert it reports no changes, preserving idempotency.

## Regression suite

- Run `cargo test agents::` for all installer, parsing, upgrade, status, and uninstall behavior.
- Run the complete Rust unit and integration suite.
- Run strict spec validation, the documentation build, dependency audit, and repository trust gate.

## Boundaries

- Verify each tool retains its native argument placeholder and file format.
- Verify Codex remains skill-only and no deprecated command file is created.
- Verify `--minimal` alone still instructs the agent to request a missing module or description.
