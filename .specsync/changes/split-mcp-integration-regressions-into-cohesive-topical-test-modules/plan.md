---
change: split-mcp-integration-regressions-into-cohesive-topical-test-modules
artifact: plan
---

# Plan

## Approach

Keep tests/integration/mcp.rs as the existing registered module and use private child modules for protocol, tools, read roots, configuration, snapshots, issues, Gradle, workspaces, and limits. Move complete original functions, attached attributes, and explanatory comments. Keep the three cross-topic helpers and shared imports at the facade, with topic-only helpers beside their callers. Every new source file stays below 1000 lines.

## Out of scope

No production source, protocol/API behavior, assertions, fixture data, request bytes, subprocess environment, security checks, platform execution gates, or root integration registration changes. Do not rewrite FIFO or socket PermissionDenied semantics. Do not fix unrelated baseline failures or suppress existing lint diagnostics. Do not publish, self-approve implementation, archive, or merge.

## Steps

1. Save the pristine source and compiled MCP test inventory; run all 72 Linux MCP tests before movement.
2. Record the draft, interview, substantive plan, semantic documentation delta, and owner-authorized scope approval.
3. Move all 77 complete tests and nine helpers into the small facade and cohesive private topical children. Preserve the cfg(windows) commands::create_windows_junction seam and all four release-ignore rendezvous guards.
4. Compare every complete declaration, signature, body, literal, and attached attribute against pristine main. Compare compiled test leaf names and the explicit child-module registrations; run the unchanged root registration guard.
5. Update MCP test-topology documentation, context, tasks, spec version, and changelog without changing behavioral requirements or public API.
6. Run focused and complete integration tests, compare exact known failure sets, and run formatting, type checks, clippy, strict spec coverage, and the pre-push gate. Keep full-trust and platform limitations explicit.
7. Obtain independent technical review of the exact patch, then use real change check --commit for a clean scoped lifecycle checkpoint. Human implementation review remains pending.

## Risks

Unregistered children can silently omit tests, so preserve root registration and compare both the source declaration inventory and compiled test names. Relocated helpers can lose cfg gates or visibility, so keep their complete declarations and inspect the Windows command-helper seam. Debug-only rendezvous must remain ignored rather than removed under release. Large fixture strings can be changed accidentally by edits, so compare complete bodies and literal bytes. No test self-reexec relies on its module-qualified name.

## Constraints consulted

Read AGENTS.md, hi/AGENTS.md and the relevant agent/change/check/plan/spec/ship intent files, the local .codex spec-sync skill, MCP spec and every companion, the MCP integration source, and the unchanged harness registration guard. Honor retained-root confinement, read-only defaults, exact JSON-RPC behavior, successful subprocess exits, FIFO-mandatory/socket-availability semantics, and all Unix/Windows/debug guards. No configured principles file exists. hi/agent.md forbids agent self-review in place of human implementation approval.
