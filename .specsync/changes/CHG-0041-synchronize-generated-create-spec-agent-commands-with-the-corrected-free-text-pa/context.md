---
change: CHG-0041-synchronize-generated-create-spec-agent-commands-with-the-corrected-free-text-pa
artifact: context
---

# Context

PR #362 corrected the installer templates in `src/agents.rs`, but it did not refresh this
repository's checked-in Claude, Cursor, and Gemini create-spec commands. Those files still instruct
agents to select the first whitespace-delimited token before classifying the input, reproducing
GitHub issue #367 for contributors using SpecSync's own project integrations.

The checked-in commands are materialized examples of the same assets installed in consumer
repositories. Allowing them to drift creates a particularly confusing failure: unit tests prove the
installer is correct while the project itself continues teaching agents the old behavior.

## Decisions

- Keep the shared constants and render functions in `src/agents.rs` as the sole source of truth.
- Add explicit before/after `--minimal` examples for both a bare module and free text so the prompt's
  intended classification order is unambiguous.
- Regenerate the three supported checked-in command assets through `install_agent`; do not hand-edit
  their prose independently.
- Add a regression that compares checked-in asset bytes with freshly rendered installer output and
  asserts the flag-order examples plus absence of the retired first-token instruction.
- Preserve Codex's existing skill-only behavior because it has no project command asset.

## Compatibility

Bare module identifiers and the `--minimal` option remain backward compatible. This fixes stale
prompt content only; command paths, placeholders, install/uninstall behavior, and CLI grammar do not
change.
