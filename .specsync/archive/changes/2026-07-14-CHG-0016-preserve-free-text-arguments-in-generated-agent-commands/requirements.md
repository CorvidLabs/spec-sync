---
change: CHG-0016-preserve-free-text-arguments-in-generated-agent-commands
artifact: requirements
---

# Requirements

### REQ-agents-002

Generated agent integrations SHALL preserve complete user intent across each tool's documented argument syntax.

Acceptance Criteria

- Create-spec guidance removes supported flags before classifying the complete remaining input.
- A complete single module identifier is preserved unchanged.
- Quoted or unquoted natural-language descriptions are classified before a kebab-case module name is derived.
- Gemini create-change guidance uses `{{args}}` and contains no `$ARGUMENTS` reference.
- Every generated skill and create-change command quotes a free-text interview answer as one positional argument.
- Reinstalling all four integrations remains deterministic and idempotent.
