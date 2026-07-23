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
expose mutating tools only when the operator explicitly enables writes.

Acceptance Criteria

- Default MCP mode omits mutating tools.
- Direct mutator calls in default mode fail before execution.
- Write mode is explicit and mutating tools cannot override the configured root.
- MCP startup opens and identity-binds the user-requested root before ambient canonicalization,
  then canonicalizes and reopens that path and requires the reopened identity to match; replacement
  during acquisition fails before JSON-RPC request dispatch.
- Read roots are lexical descendants opened only through the retained configured-root capability.
- On Windows, absolute read roots may be spelled beneath either the original startup path or its
  canonical equivalent when both were identity-bound at startup; the derived suffix is still
  opened only through the retained canonical capability, and sibling-prefix lookalikes fail.
- Configuration/metadata/cache files, manifest/autodetection paths, dependency references, module
  names/files, spec mappings, and nested symlink targets are confined before downstream filesystem
  access.
- Recursive confinement uses cumulative deterministic budgets across configured paths and spec
  mappings and honors ignored/configured exclusions.
- Traversal, configured-path, and symlink escapes fail before filesystem access.
- Project files are bounded to 8 MiB and actual project/config input to 64 MiB per operation;
  manifests are copied from the exact bytes charged during discovery.
- Selected config is acquired through verified regular-directory and regular-file capabilities
  without following symlink/reparse points or blocking on special files. Its exact bounded bytes
  pass complete checked parsing before compatibility loading; non-object JSON, invalid UTF-8,
  malformed JSON/TOML, and wrong-typed known fields fail closed for both tools and resources.
- Cargo TOML path discovery follows only semantic target, dependency, workspace-dependency,
  target-specific dependency, patch, and replacement tables; unrelated metadata `path` keys do
  not authorize filesystem inputs. Comment/escape-aware shared Gradle workspace parsing preserves
  explicitly declared inputs beneath normally ignored names, including multiline includes and
  supported `projectDir` overrides.
- Manifest-relative `..` components are resolved from the declaring manifest and accepted only
  when the normalized target remains beneath the retained server root. Confined Windows-native
  backslashes are normalized equivalently; drive, UNC, rooted, traversal, symlink, and junction
  escapes still fail before downstream access.

### REQ-mcp-003

The MCP server SHALL validate JSON-RPC tool arguments exactly and SHALL obey notification response
semantics.

Acceptance Criteria

- Unknown keys and wrong argument types return `-32602` without tool execution.
- Tool-domain failures remain `isError: true` results.
- Every notification, including unknown methods, emits no response and cannot mutate files.
- JSON-RPC lines larger than 1 MiB are drained and rejected with `-32700` before parsing; the next
  line remains independently processable.
- Generation is limited to 1,000 specs and 64 MiB, atomically publishes through retained parent
  capabilities, and rolls back only matching filesystem and exact-byte transaction identities,
  including when a filesystem immediately reuses an inode. Exact-byte identity hashing is capped
  at the generated-output limit and fails closed above it.
- Quarantine cleanup validates its retained directory identity and consumes the final directory
  capability before removal, avoiding Windows sharing violations without reopening an ambient path.
- GitHub issue verification requires explicit `GITHUB_TOKEN`, performs reads in-process without a
  provider subprocess, prepares once, globally caps/deduplicates 100 IDs, includes authentication
  and repository preflight in its 30-second deadline, and revalidates access after an apparent
  missing issue.
- GitHub issue verification treats unreadable specs, malformed or missing frontmatter, and failed
  spec discovery as inconclusive tool errors instead of silently producing a zero-reference
  success.
- Recursive discovery and `implements`/`tracks` list shapes are checked rather than lossy; walker
  failures, wrong shapes, and invalid issue IDs are inconclusive.
- Checked issue parsing uses the shared maintained real-YAML parser: duplicate keys and malformed
  YAML anywhere reject the operation; blank/null/scalar/mapping/mixed/non-positive/overflowing
  top-level known fields are invalid; comments and valid trailing commas are accepted; nested
  extension and block-scalar lookalikes are ignored; LF and CRLF delimiters are equivalent.
- MCP read diagnostics expose only a sanitized project-relative spec path and a content-free
  reason; they do not expose the server's absolute root, raw OS error text, or spec bytes.
- MCP issue diagnostic paths normalize separators only on Windows; literal Unix filename
  backslashes remain data and cannot collide with a nested path identity.
- MCP tools/resources reject invalid UTF-8, malformed JSON/TOML, and wrong-typed selected
  specs/source path selectors before compatibility loading can substitute defaults.
