---
spec: cmd_generate.spec.md
---

## User Stories

- As a developer, I want deterministic local scaffolds for uncovered modules.
- As a developer, I want batch generation for selected modules.
- As an agent integrator, I want stable JSON paths for later refinement.

## Constraints

- Existing specs are never overwritten.
- Agent integrations may refine generated markdown outside this command.

### REQ-cmd-generate-001

The generate command SHALL scaffold specs deterministically without invoking inference, network providers, credentials, or shell commands.

Acceptance Criteria
- Provider and model flags are absent.
- Default, batch, uncovered, and JSON modes retain their non-AI behavior.
- Generated paths and exit status remain machine-readable for coding agents.

