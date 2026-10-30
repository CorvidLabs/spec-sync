# Lesson bundle — split-mcp-implementation-into-cohesive-private-modules

Material for folding this change's lessons into each affected module's lesson file.
Synthesise from what actually happened below; do not restate the change description.

## What this change was

- **Title**: Split MCP implementation into cohesive private modules
- **Kind**: Refactor
- **Specs**: mcp
- **Paths**: src/mcp.rs, src/mcp, specs/mcp
- **Acceptance**: Replace textual MCP includes with ordinary private Rust modules while preserving existing production function bodies and cfg gates; keep run_mcp_server and mcp_tool_definitions as the two external MCP entry points; conserve all existing MCP tests and scan every moved source for provider-process construction; preserve protocol responses, confinement, budgets, publication, rollback and debug race barriers; pass debug unit/integration inventories, release-test compilation and strict canonical contract checks.

## Evidence

- Verification commit: `5e23800ea78c12dc9750c6817002585324c2dbcf`
- Base commit: `621a74785e3c09b2e9ebea35768e05f5772f886f`
- Verified by: `specsync check --spec mcp`

## From the change's context.md

# Context

PR810 already split the original MCP monolith into two textual include chunks. Those chunks still share one large Rust namespace. This refactor replaces them with ordinary private modules grouped by protocol, handlers, confinement, preflight, snapshots, manifest traversal and publication. The external MCP wire/API contract is unchanged.

Grok produced the local candidate; a separate GPT reviewer compared original declarations/function bodies and test inventories. That review found a release-test import regression, corrected by direct std::fs imports in the two child test modules. No contributor or primary checkout was changed. PR800 evidence and PR815 check/fix files remain outside this scope.

This package began as a complete unapproved technical draft. Leif directly approved its prepared scope on 2026-10-16, conditional on actual local testing and at least 95% engineering confidence; the native writer recorded actor 0xLeif. The direct response is preserved in the coordinator receipt ci-cleanup-human-approval.json. Implementation review, supported finalization and platform qualification still require their actual evidence.

## From the change's design.md

# Design

The facade owns only module declarations, the unchanged server/catalog entry points and the complete moved-source provider-process absence test. Protocol handles transport; tools handles dispatch; confinement owns retained-root/path checks; preflight owns input checks; snapshot and snapshot_manifest own bounded copies and workspace discovery; publication owns staged writes and rollback. Shared test fixtures are cfg(test).

Module helpers use private or pub(super) visibility. Public and pub(crate) visibility are not introduced beneath src/mcp. Production bodies/constants and cfg boundaries are conserved; only import qualification and mechanical layout change. Protocol/snapshot child tests import std::fs directly because release-profile cfg(test) cannot inherit a debug_assertions-only parent import.

Historical server_name.rs and publish_staged_file_with_verification_hooks.rs are two-line comment-only files, retained for the existing GitHub source scan. They are no longer included as Rust code. A new MCP test explicitly scans all moved implementation files, so retaining the anchors does not hide actual code from that regression check.

## From the change's testing.md

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

Before pushing, run the mandatory fast pre-push lane and committed-range Trust. An empty base..HEAD Trust result is not evidence for this change. Preserve actual risk/contract/provenance output and report any block. Leif directly approved the prepared definition scope on 2026-10-16 and the supported writer recorded actor 0xLeif. Current native qualification, independent implementation review, same-PR finalization and hosted checks still require actual evidence. Do not remove the package or weaken policy to bypass any of those gates.

## Limits

macOS local execution only. No Windows/Linux runtime or immutable release-candidate qualification is claimed. No new dependency/tool was downloaded. Tests do not prove human acceptance or a statistical95% guarantee.

## Current lifecycle follow-up

The original product source and test bytes remain unchanged from the independently reviewed and locally qualified candidate. The genuine scope approval removes the absence-of-permission condition only; no new local test run, human inspection, native review/finalization or passing hosted gate is inferred from that approval. Fresh command results will be recorded after execution.

## Where these lessons go

- `specs/mcp/mcp.3md`
