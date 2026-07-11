---
change: CHG-0003-finalize-specsync-5-0-release-consistency-and-parallel-validation
artifact: testing
---

# Testing

The focused regression allocates many effective-contract scratch paths concurrently and requires every path to be distinct. The exact PR merge fixture must pass both the focused effective-contract test and the complete suite with one and default test threads under `CI=true`, `GITHUB_ACTIONS=true`, `GITHUB_WORKSPACE`, and `GITHUB_BASE_REF`. Final evidence also includes Clippy, rustfmt, strict 100% spec ownership, the repository lane, all executable SDD examples, runtime coverage, packaged Action consumption, and the GitHub Linux/macOS/Windows matrix.

## Requirement Evidence

- `REQ-change-011`: `effective_contract_workspaces_are_unique` and `unified_gate_validates_code_against_effective_delta`.
- `REQ-agents-001`: agent install/content/idempotency tests plus the clean-project 10-artifact inventory.
- `REQ-cmd-agents-001`: CLI agent target parsing and install dispatch coverage.
- `REQ-cmd-init-002`: current-layout initialization unit and integration matrix.
- `REQ-commands-001`: strict canonical validation of the command registry.
- `REQ-ai-001`: deprecated-provider routing and trusted-command compatibility tests.
- `REQ-cli-001`: CLI lifecycle integration tests and structured output checks.
- `REQ-cli-args-001`: `change_new_collects_sdd_scope` and CLI parser tests.
