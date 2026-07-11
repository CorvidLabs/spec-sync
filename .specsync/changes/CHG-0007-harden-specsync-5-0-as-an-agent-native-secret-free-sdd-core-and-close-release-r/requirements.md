---
change: CHG-0007-harden-specsync-5-0-as-an-agent-native-secret-free-sdd-core-and-close-release-r
artifact: requirements
---

# Requirements

## REQ-exports-001

The Rust export scanner SHALL preserve every documented contract symbol across every source file listed by a spec.

Acceptance Criteria
- Regex and AST parsing include plain `pub` and crate-visible `pub(crate)` declarations.
- Narrower `pub(super)`, `pub(self)`, and `pub(in ...)` declarations remain excluded.
- A multi-file fixture matching issue #334 passes strict phantom/undocumented export validation in both parse modes.

## REQ-core-001

The SpecSync core SHALL be deterministic, agent-native, and free of embedded inference credentials or execution.

Acceptance Criteria
- The production dependency graph contains no `corvid-ai` client.
- The CLI, config schema, MCP schema, and generator contain no provider, model, API-key, base-URL, automatic source-upload, or AI-command path.
- `specsync generate` always scaffolds deterministic local templates.
- MCP, native agent skills, and slash-command integrations remain available so a coding agent can enrich the scaffold using its own trust boundary.
- Legacy AI configuration keys are ignored with migration guidance and never interpreted as credentials or commands.

## REQ-release-security-001

The 5.0 release SHALL close known repository security and CI-reporting findings before publication.

Acceptance Criteria
- The documentation site resolves all five open Astro advisories by using Astro 6.4.6 or newer.
- PR comment generation emits only the rendered report on stdout and bounds mascot context below operating-system argument limits.
- README and repository documentation explain the agent-native, secret-free architecture and removal of embedded AI flags/configuration.
- CodeQL, dependency audit, strict specs, cross-platform tests, executable SDD examples, and the packaged Action consumer pass.

## REQ-comment-001

The pull-request reporting path SHALL be protocol-clean and bounded for GitHub Actions.

Acceptance Criteria
- `specsync comment` emits only the rendered markdown report on stdout; configured verification command output cannot contaminate it.
- The rendered report is bounded below GitHub's comment-body limit with explicit truncation guidance.
- The mascot workflow passes a bounded context value without exceeding operating-system argument limits.
