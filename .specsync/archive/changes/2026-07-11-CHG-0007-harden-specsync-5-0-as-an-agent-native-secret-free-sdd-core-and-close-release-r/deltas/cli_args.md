## ADDED

### REQUIREMENT REQ-cli-args-002
The shared CLI grammar SHALL expose deterministic generation without embedded inference selection.

Acceptance Criteria
- `generate` retains deterministic module, uncovered, and batch selection.
- `generate` exposes no provider or model flags.
- Agent installation and MCP commands remain available.
