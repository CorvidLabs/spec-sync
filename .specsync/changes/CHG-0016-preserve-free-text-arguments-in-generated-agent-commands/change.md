---
id: CHG-0016-preserve-free-text-arguments-in-generated-agent-commands
state: accepted
type: bug_fix
base_commit: 60bd655c2365addc3d7a37e95f5fc20c06a746ff
---

# Preserve free-text arguments in generated agent commands

## Intent

Preserve free-text arguments in generated agent commands

## Affected Canonical Specs

- `agents`

## Acceptance Criteria

- Generated create-spec instructions strip supported flags before classifying the complete remaining input; bare identifiers remain unchanged; natural-language descriptions never become first-word module names; Gemini create-change uses its args placeholder and never ARGUMENTS; every integration quotes multi-word interview answers as one argument; installing all four integrations stays deterministic and idempotent; focused and full Rust tests pass

## No-spec Rationale

Not applicable
