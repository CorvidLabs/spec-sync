## MODIFIED

### REQUIREMENT REQ-cli-002

The CLI SHALL expose deterministic generation and agent integrations without accepting embedded model-provider or credential options.

Acceptance Criteria
- `generate` has no provider/model flags.
- MCP and agent installation commands remain available.
- Text, JSON and named generation support custom Cargo library paths and scoped npm names with truthful output and exit status.
