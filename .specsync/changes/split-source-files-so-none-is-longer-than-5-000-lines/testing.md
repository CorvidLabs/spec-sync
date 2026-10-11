---
change: split-source-files-so-none-is-longer-than-5-000-lines
artifact: testing
---

# Testing

## Requirement evidence

| Requirement | Evidence |
|-------------|----------|
| REQ-change-109 | `cargo check --bin specsync --tests` compiles the included change parts. No part exceeds 5,000 lines. |
| REQ-mcp-010 | The MCP parts are the previous `src/mcp.rs` bytes, included in order. |
| REQ-validator-043 | The validator parts are the previous `src/validator.rs` bytes, included in order. |
| REQ-manifest-019 | The manifest parts are the previous body of `src/manifest.rs`, included after the module docs. |
| REQ-github-022 | The GitHub source scan includes `src/mcp/server_name.rs` and `src/mcp/publish_staged_file_with_verification_hooks.rs`. |

`cargo check --bin specsync --tests --offline` passed.
