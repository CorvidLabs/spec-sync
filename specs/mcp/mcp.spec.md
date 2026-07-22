---
module: mcp
version: 5
status: stable
files:
  - src/mcp.rs
db_tables: []
tracks: [30]
depends_on:
  - specs/types/types.spec.md
  - specs/validator/validator.spec.md
  - specs/config/config.spec.md
  - specs/scoring/scoring.spec.md
  - specs/generator/generator.spec.md
  - specs/deps/deps.spec.md
---

# Mcp

## Purpose

Model Context Protocol (MCP) server for AI agent integration. Implements JSON-RPC 2.0 over stdio, exposing spec-sync functionality as tools callable from Claude Code, Cursor, Windsurf, and other MCP-compatible agents.

## Public API

| Function | Parameters | Returns | Description |
|----------|-----------|---------|-------------|
| `run_mcp_server` | `root: &Path, allow_write: bool` | `Result<(), String>` | Run the confined MCP server; mutating tools are exposed only when writes are explicitly enabled and root resolution failures are reported |

## Invariants

1. Protocol version is "2024-11-05".
2. Server reports tools and resources capabilities.
3. Read-only mode exposes five non-mutating tools; write mode additionally exposes
   `specsync_generate` and `specsync_init`.
4. All filesystem operations remain within the canonical configured root.
5. Read tools may select only an existing canonical descendant root.
6. Mutating tools require write mode, reject root overrides, and use the configured root.
7. Tool argument schemas and runtime validation reject unknown properties and wrong types.
8. Tool-domain errors use `isError`; JSON-RPC shape errors use protocol error objects.
9. Every notification, including unknown methods, receives no response and cannot mutate state.
10. Config/metadata/cache files and paths, manifest/autodetection paths, dependency references,
    module names/files, spec mappings, nested symlinks, and write destinations are validated against
    the canonical root before use.
11. Recursive checks canonicalize only symlinks, honor ignored/configured exclusions, and share
    deterministic cumulative bounds across configured paths and spec mappings.
12. JSON-RPC input lines are bounded to 1 MiB and oversized lines are drained before the next request.
13. Resources and deterministic generation behavior remain compatible when authorized.

## Behavioral Examples

### Scenario: Initialize MCP session

- **Given** a client sends `{"jsonrpc":"2.0","id":1,"method":"initialize"}`
- **When** the server processes the request
- **Then** responds with protocol version, capabilities, and server info

### Scenario: Call specsync_check tool

- **Given** a client sends a `tools/call` request with `name: "specsync_check"`
- **When** the server processes the request
- **Then** responds with validation results including passed/failed status, errors, and warnings

### Scenario: Default server denies mutation

- **Given** the operator starts `specsync mcp` without `--allow-write`
- **When** a client lists tools or directly calls `specsync_init` or `specsync_generate`
- **Then** mutators are absent from the list and direct calls fail before filesystem access

### Scenario: Confined child-root read

- **Given** a read tool supplies an existing child directory as `root`
- **When** the path canonicalizes beneath the configured server root
- **Then** the tool reads that child project; outside, nonexistent, traversal, and symlink-escape roots fail

### Scenario: Reject an indirect configured escape

- **Given** a project config, manifest workspace, dependency/cache/schema reference, module
  definition, spec mapping, or nested symlink resolves outside the server root
- **When** any MCP tool or resource attempts to load the project
- **Then** the operation returns an error before downstream discovery, reading, generation, or initialization

### Scenario: List available resources

- **Given** a client sends `{"jsonrpc":"2.0","id":2,"method":"resources/list"}`
- **When** the server processes the request
- **Then** responds with 4 static resources and 1 resource template

### Scenario: Read a spec by module name

- **Given** a client sends `{"jsonrpc":"2.0","id":3,"method":"resources/read","params":{"uri":"specsync:///specs/auth"}}`
- **When** the module "auth" exists in the project
- **Then** responds with the full spec content as text/markdown

### Scenario: Unknown method

- **Given** a client sends a request with `method: "unknown/method"` and an `id`
- **When** the server processes the request
- **Then** responds with JSON-RPC error code -32601 "Method not found"

## Error Cases

| Condition | Behavior |
|-----------|----------|
| Malformed JSON input | JSON-RPC error -32700 "Parse error" |
| JSON-RPC input line exceeds 1 MiB | JSON-RPC error -32700; the line is drained and the next request remains processable |
| Unknown method with id | JSON-RPC error -32601 "Method not found" |
| Unknown tool name | Tool error: "Unknown tool: {name}" |
| Mutating tool in read-only mode | Tool error requiring `--allow-write`; no tool execution |
| Mutating tool with a per-call `root` | JSON-RPC error -32602; the server root remains authoritative |
| Non-object params/arguments, wrong argument type, or unknown key | JSON-RPC error -32602 before tool execution |
| Read root outside the server root, nonexistent, traversing, or symlink-escaped | Tool error with `isError: true`; no outside mutation |
| Configured, manifest-derived, dependency/cache/schema, module, spec-mapping, or nested-symlink path escapes the root | Tool/resource error before downstream filesystem access; outside bytes remain unchanged |
| Cumulative confinement or manifest preflight exceeds its deterministic entry bound | Tool/resource error before downstream filesystem access |
| Server root cannot be resolved | Server exits nonzero and writes an actionable diagnostic to stderr |
| Unknown resource URI | JSON-RPC error -32602 "Unknown resource URI: {uri}" |
| Spec module not found | JSON-RPC error -32602 "No spec found for module: {name}" |
| No spec files found | Tool error with suggestion to run `specsync generate` |
| Retired AI/provider/model/credential/endpoint/command argument | JSON-RPC error -32602 with migration guidance; supplied values are not echoed |
| Any parsed request without `id` | No response and no dispatch |
| stdin EOF | Server exits gracefully |

## Dependencies

### Consumes

| Module | What is used |
|--------|-------------|
| config | `load_config`, `detect_source_dirs` |
| validator | `validate_spec`, `find_spec_files`, `compute_coverage`, `get_schema_table_names` |
| generator | `generate_specs_for_unspecced_modules_paths` |
| scoring | `score_spec`, `compute_project_score` |
| parser | `parse_frontmatter` |
| types | `SpecSyncConfig` |
| deps | `build_dep_graph`, `validate_deps`, `topological_sort` |

### Consumed By

| Module | What is used |
|--------|-------------|
| main | `run_mcp_server` (via `mcp` subcommand) |

## Change Log

| Date | Change |
|------|--------|
| 2026-06-07 | Tighten generated-spec test assertion for MCP tool coverage |
| 2026-04-10 | Add MCP resources: specs list, spec by module, dependency graph, config, coverage |
| 2026-03-25 | Initial spec |
| 2026-07-11 | CHG-0007-harden-specsync-5-0-as-an-agent-native-secret-free-sdd-core-and-close-release-r: Harden SpecSync 5.0 as an agent-native, secret-free SDD core and close release regressions |
| 2026-07-21 | CHG-0062: Make MCP read-only by default with explicit confined writes, exact argument validation, and notification-safe dispatch |
| 2026-07-22 | CHG-0062-harden-mcp-root-confinement-write-authorization-argument-validation-and-notif: Harden MCP root confinement, write authorization, argument validation, and notification semantics for issue 414 |
