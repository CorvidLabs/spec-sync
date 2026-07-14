---
id: CHG-0041-synchronize-generated-create-spec-agent-commands-with-the-corrected-free-text-pa
state: implementing
type: bug_fix
base_commit: 44aab88697a141e65b0a16d4bc4fd8704972d435
---

# Synchronize generated create-spec agent commands with the corrected free-text parser guidance and prevent checked-in asset drift

## Intent

Synchronize generated create-spec agent commands with the corrected free-text parser guidance and prevent checked-in asset drift

## Affected Canonical Specs

- `agents`

## Acceptance Criteria

- Checked-in Claude, Cursor, and Gemini create-spec command assets remove standalone --minimal flags before classifying the complete remaining input and contain no first-token module-name guidance.
- Bare module names remain unchanged while quoted or unquoted natural-language descriptions derive a meaningful deterministic kebab-case slug instead of using the first word.
- Generated guidance explicitly covers --minimal before and after both bare-module and free-text examples.
- Generator regression tests prove the checked-in assets match current rendered templates and reinstalling remains deterministic and idempotent.

## No-spec Rationale

Not applicable
