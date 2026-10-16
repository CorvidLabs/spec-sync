---
change: split-mcp-implementation-into-cohesive-private-modules
artifact: testing
---

# Testing

## Local evidence

- Existing MCP baseline:123 unit tests and72 integration tests; original source-declared test inventory conserved, including cfg-only cases.
- Corrected candidate:124 MCP unit tests and72 unchanged MCP integration tests pass.
- Full local tests:2583 unit tests pass; the complete462-case integration suite passes after removing inherited CI/forced-color overrides. The original combined lane failed on environment-sensitive assertions; its failed receipt is retained. This is qualification across separately captured stages, not a claim that that lane invocation succeeded.
- Every remaining native verify stage passes: release build, source-binary strict contract check,64 release-candidate validator controls and29 required-CI controls. Format, clippy and type steps passed before the test stage.
- Latest released SpecSync6.1.1 strict check passes all62 specs with zero warnings and100% source coverage. Score average and mcp score are100/100.
- Release test compilation: cargo check --release --tests --bin specsync --offline failed101 before the import correction, then passed0. Release-profile test execution is not claimed.
- Independent GPT refactor review and narrow cfg correction review are complete. Human review is pending.

## Publication gates

Before pushing, run the mandatory fast pre-push lane and committed-range Trust. An empty base..HEAD Trust result is not evidence for this change. Preserve actual risk/contract/provenance output and report any block. Hosted checks, human definition approval, acceptance and same-PR finalization are pending; do not remove this draft or weaken policy to bypass them.

## Limits

macOS local execution only. No Windows/Linux runtime or immutable release-candidate qualification is claimed. No new dependency/tool was downloaded. Tests do not prove human acceptance or a statistical95% guarantee.
