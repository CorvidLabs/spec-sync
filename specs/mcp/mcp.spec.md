---
module: mcp
version: 15
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
4. The requested root is opened and identity-bound before canonicalization; the canonical path is
   reopened and must identify the same directory, so startup replacement cannot redirect authority.
5. Read tools lexically validate a relative descendant and open it only through the retained server
   root capability, so replacement of the ambient root path cannot redirect selection. On Windows,
   an absolute child may use either the original startup spelling or the canonical spelling of that
   same identity-bound root; only its lexical suffix is consumed through the retained canonical
   capability, and sibling-prefix lookalikes remain rejected.
6. Mutating tools require write mode, reject root overrides, and use the configured root.
7. Tool argument schemas and runtime validation reject unknown properties and wrong types.
8. Tool-domain errors use `isError`; JSON-RPC shape errors use protocol error objects.
9. Selected configuration and recognized manifest inputs are first acquired as retained regular
   file handles through explicit no-follow, non-blocking opens. Opened-handle metadata and identity
   remain authoritative across bounded reads; path observations must resolve to that identity, so
   links/reparse points, special files, and replacement identities fail before bytes are parsed.
10. Every notification, including unknown methods, receives no response and cannot mutate state.
11. Config/metadata/cache files and paths, manifest/autodetection paths, dependency references,
    module names/files, spec mappings, nested symlinks, and write destinations are validated against
    the canonical root before use.
12. Recursive checks canonicalize only symlinks, honor ignored/configured exclusions, and share
    deterministic cumulative bounds across configured paths and spec mappings.
13. JSON-RPC input lines and responses are bounded to 1 MiB, and request IDs are bounded to 4 KiB.
14. Generated output is limited to 1,000 specs and 64 MiB, preflighted before mutation, staged and
    synced beside each destination, and atomically published without overwriting existing files;
    retained parent capabilities and filesystem-plus-content identities preserve replacements at public transaction
    paths. Empty parent directories created during a failed batch may remain because no portable
    create-and-open directory primitive can prove ownership across a concurrent replacement.
    Private quarantine cleanup consumes its final directory capability before removal so Windows
    does not retain a sharing-blocking handle.
    Processes already authorized to mutate the server root must not race private
    `.specsync-mcp-stage-*` or `.specsync-mcp-quarantine-*` names.
15. Explicit root-wide and manifest-derived inputs remain present in bounded snapshots even when
    they cross normally ignored directory names; ignored or configured-exclusion symlink names
    are skipped before following targets unless an explicit configured input names them or a descendant;
    manifest discovery parses Cargo workspace
    membership as TOML plus comment/escape-aware shared Gradle settings; snapshots copy the exact
    manifest bytes charged to the shared cumulative byte budget.
    Cargo path discovery follows only semantic target, dependency, workspace-dependency,
    target-specific dependency, patch, and replacement tables; unrelated metadata named `path`
    is ignored. Manifest-relative parent components and confined Windows-native backslashes are
    normalized from the declaring manifest and accepted when their resolved target remains beneath
    the server root; drive, UNC, rooted, traversal, symlink, and junction escapes fail.
16. GitHub issue verification requires explicit `GITHUB_TOKEN`, performs read/list/verify requests
    in-process without a provider subprocess, prepares once, globally deduplicates at most 100 IDs,
    includes authentication/preflight in the 30-second batch bound, and revalidates repository
    access before accepting not-found; provider failures remain inconclusive tool errors rather
    than successful empty/not-found results. Spec discovery, reads, and frontmatter parsing must
    also complete through checked traversal and the shared maintained real-YAML issue parser.
    Duplicate keys or malformed YAML anywhere, plus blank/null/wrong-shaped top-level
    `implements`/`tracks`, make verification inconclusive. Comments and valid trailing commas are
    accepted; nested extension and block-scalar lookalikes are ignored. Read diagnostics use only a
    sanitized relative spec path and a content-free reason, never host-absolute paths, raw OS
    errors, or spec bytes. Windows diagnostic separators render as `/`; Unix literal backslashes
    remain filename data and are not conflated with nested paths.
17. A selected MCP configuration is acquired through verified regular-directory and regular-file
    capabilities with no symlink/reparse traversal, non-blocking open, identity checks, and the
    normal per-file bound. Its exact retained bytes pass the complete checked config parser before
    any compatibility loader runs. Non-object JSON, invalid UTF-8, malformed JSON/TOML, and
    wrong-typed known fields make tools and resources inconclusive instead of silently falling
    back to an empty/default project.

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
| JSON-RPC request ID exceeds 4 KiB | JSON-RPC error -32600 with a null ID; no dispatch |
| Unknown method with id | JSON-RPC error -32601 "Method not found" |
| Unknown tool name | Tool error: "Unknown tool: {name}" |
| Mutating tool in read-only mode | Tool error requiring `--allow-write`; no tool execution |
| Mutating tool with a per-call `root` | JSON-RPC error -32602; the server root remains authoritative |
| Non-object params/arguments, wrong argument type, or unknown key | JSON-RPC error -32602 before tool execution |
| Read root outside the server root, nonexistent, traversing, symlink-escaped, or selected after ambient root replacement | Tool error with `isError: true`; retained authority never follows the replacement path |
| Configured, manifest-derived, dependency/cache/schema, module, spec-mapping, or nested-symlink path escapes the root | Tool/resource error before downstream filesystem access; outside bytes remain unchanged |
| Semantic Cargo sibling path such as `../b` or `..\b` normalizes inside the root | Accepted and included in the bounded snapshot; drive, UNC, rooted, traversal, symlink, and junction escapes still fail |
| Unrelated Cargo metadata contains a `path` key | Ignored for snapshot input discovery; only semantic Cargo target/workspace/dependency path tables authorize an input |
| Cumulative confinement or manifest preflight exceeds its deterministic entry bound | Tool/resource error before downstream filesystem access |
| Selected config or recognized manifest is a FIFO, device, symlink/reparse point, or replaced identity | Tool/resource error before parsing; the server does not block or consume replacement bytes |
| Generation exceeds 1,000 specs, 64 MiB, or its response budget | Tool error before publishing project files |
| Generated destination exists, a public parent path is replaced, or a staged batch cannot publish completely | Tool error; identity-bound cleanup preserves public replacements; an empty parent created by the failed batch may remain |
| Private quarantine cleanup on Windows | Final retained directory handle is consumed before removal; successful init/generate does not fail with a sharing violation |
| GitHub issue provider tree, authentication, repository recheck, timeout, malformed output, transport failure, traversal error, unreadable spec, malformed frontmatter, or invalid `implements`/`tracks` shape | Inconclusive tool error; no trustworthy zero-count or not-found result; read diagnostics remain relative and content-free |
| Checked issue frontmatter has duplicate/global malformed YAML or blank/null/wrong-shaped known fields | Inconclusive tool error; comments/trailing commas remain valid and nested extension/block-scalar lookalikes do not become references |
| Server root cannot be resolved | Server exits nonzero and writes an actionable diagnostic to stderr |
| Unknown resource URI | JSON-RPC error -32602 "Unknown resource URI: {uri}" |
| Spec module not found | JSON-RPC error -32602 "No spec found for module: {name}" |
| No spec files found | Tool error with suggestion to run `specsync generate` |
| Selected config is linked/reparse-backed, non-regular, blocking, replaced, invalid UTF-8, structurally invalid or malformed JSON/TOML, or has wrong-typed known fields | Tool/resource error before compatibility loading; no path traversal, indefinite block, or default-path false success |
| Retired AI/provider/model/credential/endpoint/command argument | JSON-RPC error -32602 with migration guidance; supplied values are not echoed |
| Any parsed request without `id` | No response and no dispatch |
| stdin EOF | Server exits gracefully |

## Dependencies

### Consumes

| Module | What is used |
|--------|-------------|
| config | `load_config`, `detect_source_dirs`, `parse_config_content_checked` |
| validator | `validate_spec`, `find_spec_files`, `compute_coverage_checked`, `get_schema_table_names` |
| generator | `generate_specs_for_unspecced_modules_paths` |
| scoring | `score_spec`, `compute_project_score` |
| parser | `parse_frontmatter`, `parse_checked_issue_references` |
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
| 2026-07-22 | CHG-0063: Identity-bind roots and quarantine rollback, parse Cargo TOML and checked Gradle inputs, normalize Windows roots, and use bounded in-process GitHub verification |
| 2026-07-22 | CHG-0063 defensive review: Skip ignored-name symlinks before traversal and compare Windows root components with native ordinal Unicode case semantics |
| 2026-07-22 | CHG-0063 CI follow-up: Bind staged and rollback identities to exact bytes so immediate Unix inode reuse cannot authorize a replacement, with fail-closed bounded hashing |
| 2026-07-22 | CHG-0063 compatibility follow-up: Accept manifest-relative sibling paths that normalize inside the retained MCP root, reject true escapes, and consume private quarantine handles before Windows removal |
| 2026-07-22 | CHG-0063 independent-review follow-up: Restrict Cargo path discovery, normalize confined Windows-native paths, make checked issue discovery/field parsing fail closed with relative content-free diagnostics, and repair Windows fixtures |
| 2026-07-22 | CHG-0063 final adversarial follow-up: Share maintained real-YAML checked issue parsing, reject duplicate/global malformed YAML and blank/null/wrong shapes, and preserve valid comments/trailing commas |
| 2026-07-22 | CHG-0063 Windows CI follow-up: Accept absolute children beneath the identity-bound startup root when Windows expands an 8.3 alias, while continuing to open only through the retained canonical capability and reject sibling-prefix lookalikes |
| 2026-07-22 | CHG-0063 adversarial follow-up: Preserve literal Unix backslashes in MCP issue diagnostic identities while normalizing separators only on Windows |
| 2026-07-22 | CHG-0063 final config follow-up: Validate selected bounded config bytes and path-selector types before compatibility loading |
| 2026-07-22 | CHG-0063 no-follow config follow-up: Reject linked/reparse, non-regular, blocking, replaced, and structurally invalid selected configs through an identity-verified bounded snapshot and the complete checked parser |
| 2026-07-22 | CHG-0063 final agent-review follow-up: Bind selected configs to their pre-open identity and reject special-file manifests before blocking reads |
| 2026-07-22 | CHG-0063 retained-handle follow-up: Acquire selected configs and manifests with no-follow, non-blocking handles on every platform; validate opened metadata and reject path replacement before and after bounded reads |
