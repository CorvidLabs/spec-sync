---
spec: mcp.spec.md
---

## User Stories

- As a Claude Code user, I want spec-sync available as an MCP server so that I can check, generate, and score specs directly from my AI assistant
- As a Cursor user, I want MCP tools for spec-sync so that spec validation is integrated into my AI-powered editing workflow
- As an AI agent developer, I want programmatic access to spec-sync over JSON-RPC so that I can build spec validation into automated workflows
- As an AI agent, I want to read specs, the dependency graph, config, and coverage as MCP resources so that I can inspect project state without invoking a tool
- As a developer, I want the MCP server to run over stdio so that it works with any MCP-compatible client without network configuration
- As an operator, I want MCP mutation disabled by default and confined to one project root when enabled

## Acceptance Criteria

- Implements JSON-RPC 2.0 over stdio
- Protocol version "2024-11-05" returned in initialize response; `initialize` advertises both `tools` and `resources` capabilities
- Five read tools are exposed by default; `specsync_generate` and `specsync_init` are exposed only with `--allow-write`
- Four resources exposed (`specsync:///specs`, `specsync:///graph`, `specsync:///config`, `specsync:///coverage`) plus one resource template (`specsync:///specs/{module}`)
- `specsync_generate` creates deterministic local scaffolds with no inference schema
- Retired AI/provider/model/credential/base-URL/command arguments return an explicit error without echoing values
- Tool-domain errors return `isError: true`; malformed tool-call shapes and arguments return JSON-RPC -32602
- Resource read errors return JSON-RPC error -32602 (unknown URI, module not found)
- Malformed JSON returns JSON-RPC error -32700 "Parse error"
- Unknown request methods return JSON-RPC error -32601 "Method not found"
- Unknown tool name returns tool-level error "Unknown tool: {name}"
- Every notification receives no response and is suppressed before dispatch, including known tools and unknown methods
- `ping` method returns empty result
- Read tools accept only lexical descendants opened through the retained server-root capability
- Mutating tools reject per-call roots and always use the configured server root
- Configuration and metadata files, configured project paths, manifest workspaces, dependency and
  cache/schema references, module names, spec file mappings, and nested symlink targets are
  validated against the canonical server root before a tool or resource runs
- Recursive confinement and autodetection preflights are bounded and honor ignored/excluded directories
- Declared Cargo workspace members and Node workspace patterns are charged before expansion;
  normalized workspace nodes are deduplicated and completed results are reused.
- Zero-config manifest/source autodetection starts only after the project root is retained and never
  trusts an ambient manifest replacement as source-directory authority.
- JSON-RPC input is limited to 1 MiB per line; oversized lines are drained and rejected before parsing
- stdin EOF triggers graceful exit

## Constraints

- Must conform to the MCP specification — no custom protocol extensions
- All output must be valid JSON (no ANSI colors, no stderr mixing into protocol)
- Must be stateless — each tool call is independent

## Out of Scope

- MCP prompts (tools and resources are implemented; prompts are not)
- HTTP/SSE transport (stdio only)
- Network authentication (stdio capability authorization is enforced by `--allow-write`)
- Streaming partial results during long operations
- Server-side state between calls (each invocation reloads config from scratch)

### REQ-mcp-001

The MCP generate tool SHALL scaffold deterministically and delegate enrichment to its connected coding agent.

Acceptance Criteria
- The tool schema contains no AI/provider argument.
- Authorized deterministic generation and resource behavior remain stable.

### REQ-mcp-002

The MCP server SHALL confine every filesystem operation to its configured project root and SHALL
bound project-controlled inputs before downstream parsing.

Acceptance Criteria

- Absolute outside roots are rejected lexically before metadata or symlink resolution.
- The real MCP CLI passes the user-requested root unchanged into startup, which opens and
  identity-binds it before ambient canonicalization, then canonicalizes/reopens it and requires
  the same identity before JSON-RPC dispatch.
- The canonical server root is retained as a directory capability; reads execute from bounded
  snapshots and writes resolve only through the capability.
- Project-controlled Git metadata is never used for MCP repository auto-detection; issue checks
  require explicit `github.repo` configuration.
- Every case variant of `.git` is rejected as a configured input, read-root component, and
  snapshot entry before Git metadata can become operation authority.
- Every read-root component is opened as an identity-checked regular directory without
  symlink/reparse traversal, so an alias cannot redirect authority into `.git`.
- Project inputs are bounded to 8 MiB per file and 64 MiB of actual file/config bytes cumulatively;
  explicitly configured normally ignored roots remain eligible.
- Ignored and configured-exclusion symlink names are skipped before their targets are followed,
  unless an explicit configured input names the path or a descendant; broad ancestor inputs do
  not override configured exclusions.
- Manifest discovery parses Cargo workspace membership as TOML and comment/escape-aware Gradle
  settings, charges deduplicated manifest bytes to the shared cumulative input budget, and copies
  the exact preflight buffers.
- Every declared Cargo member and Node workspace pattern consumes bounded expansion work;
  snapshot collection and preflight charge declarations before deduplication, normalize patterns,
  bases, workspace paths, and manifest nodes, and reuse completed results.
- Zero-config manifest/source detection begins after root capability retention and accepts only
  retained manifest observations as source-directory authority.
- Before any manifest-derived traversal, every present `build.gradle`, `build.gradle.kts`,
  `settings.gradle`, and `settings.gradle.kts` candidate is opened no-follow and non-blocking
  through the retained root capability, required to remain a regular non-link file with stable
  identity, and bounded to 4 MiB. An unsafe or oversized candidate fails the operation even when
  another candidate would otherwise be selected.
- All four Gradle build/settings names are acquired once through retained no-follow, non-blocking
  regular-file handles with the shared 4 MiB limit before parsing or source probing. Special,
  linked/reparse-backed, replaced, oversized, or invalid-UTF-8 inputs reject tools and resources;
  generic snapshot traversal never reopens the preloaded paths.
- Cargo path discovery follows only semantic target, dependency, workspace-dependency,
  target-specific dependency, patch, and replacement tables; unrelated metadata `path` keys are
  ignored.
- Unix symlink and Windows junction/reparse-point escapes fail before outside access.
- Windows absolute-root components are compared with native ordinal Unicode ignore-case semantics
  without lossy UTF-8 conversion.
- Absolute Windows children may use either original or canonical startup spelling after startup
  identity-binds both spellings; only the relative suffix is opened through the retained canonical
  capability, and sibling-prefix lookalikes are rejected.
- Selected config is opened non-blocking through verified regular-directory and regular-file
  capabilities, rejects symlink/reparse and special-file paths, requires the opened identity to
  match the pre-open inspected identity before reading, rechecks after the bounded read, and passes
  the exact retained bytes through complete checked parsing before compatibility loading.
  Recognized snapshot manifests follow the same non-blocking regular-file and pre-open identity
  rule. Non-object JSON, invalid UTF-8, malformed JSON/TOML, and wrong-typed known fields make every
  tool/resource inconclusive rather than selecting defaults.
- Generic project files used by tools and resources follow the same no-follow, non-blocking,
  retained-handle acquisition. Path and opened-handle identity must agree before and after the
  bounded read; FIFO/socket/device, link/reparse, and regular replacement races fail without
  consuming replacement bytes or returning partial snapshots.
- Manifest-relative Cargo paths may normalize `..` across sibling crates when the normalized result
  remains beneath the retained root. Confined Windows-native backslashes normalize equivalently;
  drive, UNC, rooted, traversal, canonical, symlink, and junction escapes remain rejected.

### REQ-mcp-003

The MCP server SHALL validate JSON-RPC envelopes and arguments before dispatch and SHALL fail closed
when protocol output or deterministic generation cannot complete safely.

Acceptance Criteria

- Invalid request envelopes return `-32600` before dispatch, including in write-enabled mode.
- Request IDs accept only non-null strings or integers; null and fractional IDs return `-32600`.
- Initialize requires typed protocol version, capabilities, and client name/version fields;
  malformed negotiation returns `-32602`.
- Resource arguments are exact-schema validated with `-32602` failures.
- Responses are bounded to 1 MiB; bounded request IDs are preserved while oversized IDs safely fall
  back to `null`, and transport failures are surfaced.
- Generation destination collisions and incomplete writes return tool errors instead of success;
  partial multi-file output is rolled back through retained parent capabilities after filesystem
  identity checks.
- Staged publication reopens the public parent without links/reparse points and rejects an identity
  change before and after linking.
- A failed post-link parent check cleans the exact quarantined identity before returning and
  reports cleanup failure without removing a replacement.
- Destination-open, destination-identity, destination-mismatch, and public-parent failures after
  the hard link all use the same exact cleanup path.
- One transaction-wide retained root capability is shared across staged outputs rather than
  retaining one additional root handle per generated spec.
- Selected read-root component identities are retained and the complete route is reopened and
  revalidated before a successful tool/resource response.
- GitHub issue verification requires explicit `GITHUB_TOKEN`, runs read/list/verify requests
  in-process without a provider subprocess, globally deduplicates/caps IDs, includes
  authentication/preflight in elapsed-time bounds, revalidates access after apparent absence, and
  treats provider failures as inconclusive.
- Failed spec discovery, unreadable specs, and malformed or missing frontmatter make MCP issue
  verification inconclusive instead of producing a successful zero-reference result.
- Checked issue parsing uses maintained real-YAML semantics: duplicate/global malformed YAML and
  blank/null/wrong-shaped known fields fail closed; comments/trailing commas remain valid; nested
  extension and block-scalar lookalikes are ignored; LF and CRLF frontmatter delimiters are
  accepted equivalently.
- Issue diagnostic paths normalize separators only on Windows; literal Unix backslashes remain
  filename data and cannot collide with a nested-path identity.
- Selected config validation runs before allow-empty tools/resources can report an empty/default
  project.
- Windows transaction cleanup consumes the final quarantine directory capability before name-based
  removal so init, generation, and collision rollback do not fail with sharing violations.

