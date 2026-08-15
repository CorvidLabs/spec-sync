---
module: mcp
version: 27
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
4. All filesystem operations remain within the retained canonical server-root capability,
   including read-root selection and generated-file rollback after ambient path replacement.
5. Absolute outside read roots are rejected before filesystem probing; in-root candidates must be
   existing canonical descendants and must not contain a `.git` component in any ASCII case.
6. Mutating tools require write mode, reject root overrides, and use the configured root.
7. Complete JSON-RPC envelopes, tool arguments, and resource arguments are exactly validated before
   dispatch.
8. Tool-domain errors use `isError`; JSON-RPC shape errors use protocol error objects.
9. Every valid notification, including unknown methods, receives no response and cannot mutate.
10. Project-controlled Git metadata is not used for MCP issue-repository discovery and every case
    variant of `.git` is excluded from read-root authority, configuration inputs, and snapshots.
11. Project files are bounded to 8 MiB each and actual configured operation inputs, including config
    files, to 64 MiB cumulatively; explicit normally ignored roots remain eligible.
12. JSON-RPC input and output are bounded to 1 MiB; oversized input is drained and oversized output
    becomes a compact `-32603` response with a bounded ID or `null` fallback.
13. Generation collisions and incomplete writes are failures; public transaction paths are
    identity-bound and replacements are preserved. Post-link parent failure cleans the exact
    quarantined staged identity, and the batch shares one retained root capability across outputs.
    Empty parents created by failed batches may remain, and same-user mutation of private
    transaction names is outside the MCP caller/path confinement threat boundary.
14. Snapshot scoring reports Git freshness unavailable and withholds freshness credit.
15. Manifest-derived inputs remain visible across fixed ignores, including TOML Cargo workspaces and
    comment/escape-aware Gradle settings, and snapshots copy exact bytes charged to the operation
    budget.
16. Issue verification requires explicit `GITHUB_TOKEN`, runs in-process without provider
    subprocesses, prepares once, globally deduplicates at most 100 IDs, revalidates post-404 access,
    includes authentication/preflight in the 30-second batch bound, and revalidates repository
    access before accepting not-found; provider failures remain inconclusive tool errors rather
    than successful empty/not-found results.
17. Snapshot traversal skips ignored or configured-exclusion symlink names before following target
    metadata unless an explicit configured input names them or a descendant; broad ancestor inputs
    do not override configured exclusions.
18. Windows absolute-root suffix derivation uses native path components and ordinal Unicode
    ignore-case comparison, accepts original/canonical spellings only after startup identity
    binding, and rejects sibling-prefix lookalikes.
19. Cargo filesystem inputs come only from semantic target, dependency, workspace-dependency,
    target-specific dependency, patch, and replacement tables; unrelated metadata `path` keys are
    ignored. Manifest-relative Cargo paths and confined Windows-native backslashes normalize only
    while the result remains beneath the retained root; drive, UNC, rooted, traversal, canonical,
    symlink, and junction escapes are rejected.
20. Windows transaction cleanup consumes the final quarantine directory capability before
    name-based removal, preserving init, generation, and collision rollback behavior without
    weakening identity checks.
21. MCP issue verification fails inconclusive when spec discovery, bounded reads, or frontmatter
    parsing cannot complete; unreadable or malformed specs are never silently omitted.
22. MCP issue fields are parsed by the shared maintained real-YAML checked parser; duplicate/global
    malformed YAML and invalid known shapes fail closed while valid comments/trailing commas and
    non-authoritative nested/block-scalar data remain supported.
23. The real MCP CLI preserves the user-requested root until startup opens and identity-binds it;
    canonicalization and capability reopening happen afterward, and any identity change fails
    before JSON-RPC dispatch.
24. MCP issue diagnostic paths normalize separators only on Windows; Unix literal backslashes
    remain filename data rather than hierarchy.
25. Selected config is acquired through no-follow, non-blocking, identity-verified regular-file
    snapshots and validated from exact bounded bytes with the complete checked parser before
    compatibility loading; non-object/malformed/invalid-UTF-8/wrong-typed configurations fail
    tools and resources closed.
26. Selected config and recognized manifests are acquired through explicit no-follow, non-blocking
    retained regular-file handles. Opened-handle metadata and native identity remain authoritative
    through bounded reads; later path observations must match on Windows and Unix.
27. All four recognized Gradle build/settings candidates are preflighted through retained handles
    with a 4 MiB per-file ceiling before manifest-derived traversal; no unsafe unselected candidate
    is silently ignored.
28. Every present Gradle build/settings variant is preflighted at 4 MiB before settings parsing or
    manifest-derived source probing, charged/copied from exact retained bytes once, and excluded
    from generic snapshot reopening.
29. Generic MCP project files use no-follow, non-blocking, identity-continuous retained reads for
    both tools and resources; special/link/replacement races fail without attacker-byte
    consumption or partial output.
30. Unix verification always exercises FIFO rejection and exercises socket rejection when the
    host permits socket fixture creation; host-level `PermissionDenied` marks only that fixture
    unavailable rather than failing before the security assertion.
31. Cargo/Node workspace expansion is bounded independently of retained-byte uniqueness and reuses
    completed normalized nodes; snapshot collection and preflight both charge declarations before
    deduplication.
32. Zero-config source selection consumes retained configuration/manifest observations after root
    retention.
33. Recursive snapshot traversal records sibling identities before sequential capability opens,
    bounding live directory handles by depth while preserving replacement detection.
34. Object-form Node workspaces require `packages`, and recognized nested package manifests are
    bounded and strictly parsed before tools/resources can report success.

## Behavioral Examples

### Scenario: Initialize MCP session

- **Given** a client sends `initialize` with a non-empty `protocolVersion`, object
  `capabilities`, and `clientInfo.name`/`clientInfo.version`
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
- **Then** the tool reads that child project; outside, nonexistent, traversal, symlink-escape, and
  `.git` metadata roots fail

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

### Scenario: Generic project input is replaced during snapshot

- **Given** a tool or resource retains a regular project file, then its pathname is replaced by a
  FIFO, socket, symlink/reparse point, or different regular file
- **When** the bounded project snapshot continues
- **Then** the operation fails inconclusively without blocking, parsing replacement bytes, or
  returning a partial tool/resource result

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
| Missing or wrong-typed initialize negotiation parameters | JSON-RPC error -32602; no successful handshake |
| Read root outside the server root, nonexistent, traversing, symlink-escaped, or selected after ambient root replacement | Tool error with `isError: true`; retained authority never follows the replacement path |
| Read root contains a `.git` component in any ASCII case | Tool error before opening the operation root or reading project-controlled Git metadata |
| Selected read-root route is replaced after acquisition | Tool/resource error at final route revalidation; no detached-directory success |
| Configured, manifest-derived, dependency/cache/schema, module, spec-mapping, or nested-symlink path escapes the root | Tool/resource error before downstream filesystem access; outside bytes remain unchanged |
| Semantic Cargo sibling path such as `../b` or `..\b` normalizes inside the root | Accepted and included in the bounded snapshot; drive, UNC, rooted, traversal, symlink, and junction escapes still fail |
| Unrelated Cargo metadata contains a `path` key | Ignored for snapshot input discovery; only semantic Cargo target/workspace/dependency path tables authorize an input |
| Cumulative confinement or manifest preflight exceeds its deterministic entry bound | Tool/resource error before downstream filesystem access |
| Cargo/Node workspace expansion exceeds its declared-work bound or repeats a completed normalized node | Tool/resource error on budget exhaustion; otherwise completed discovery is reused without subtree replay |
| Selected config or recognized manifest is a FIFO, device, symlink/reparse point, or replaced identity | Tool/resource error before parsing; the server does not block or consume replacement bytes |
| Generic project input is a FIFO/socket/device, link/reparse point, or is replaced across its retained read | Tool/resource error before downstream parsing; the server does not block, consume attacker bytes, or return a partial snapshot |
| Object-form Node workspaces omit `packages`, or a nested `package.json` is malformed/non-object/wrong-shaped | Tool/resource error before validation can report success |
| A valid snapshot has more sibling directories than the process descriptor limit | Siblings are opened sequentially; retained directory handles remain bounded by traversal depth |
| Generation exceeds 1,000 specs, 64 MiB, or its response budget | Tool error before publishing project files |
| Generated destination exists, post-link destination verification fails, a public parent path is replaced before/after linking, or a staged batch cannot publish completely | Tool error; identity-bound cleanup removes exact staged/quarantined bytes and preserves public replacements; an empty parent created by the failed batch may remain |
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
| 2026-07-23 | v16 / CHG-0063 final security rereview: Preflight every Gradle build/settings variant once through the retained no-follow reader, enforce 4 MiB before parsing/probing, and reject special, linked, replaced, or oversized inputs for tools and resources |
| 2026-07-23 | v17 / CHG-0063 post-review hardening: Apply no-follow, non-blocking, before/opened/after identity continuity to every generic project file used by MCP tools and resources |
| 2026-07-23 | v18 / CHG-0063 verification portability: Preserve FIFO coverage and execute socket assertions where the Unix host permits socket fixtures without making restricted sandboxes fail before the security assertion |
| 2026-07-23 | v19 / CHG-0063 exact-head review remediation: Charge Cargo/Node declarations before deduplication, reuse normalized completed workspace nodes, and retain zero-config manifest/source authority before autodetection |
| 2026-07-24 | v20 / CHG-0063 independent rereview remediation: Bind selected-config parent directories to pre-open identities, revalidate their complete retained edge chain after reads, and reject authority-bearing recursive snapshot directory replacement |
| 2026-07-24 | v21 / CHG-0063 exact-head rereview remediation: Bound recursive snapshot handles by depth, require object-form Node workspace packages, and strictly parse nested package manifests |
| 2026-07-24 | v22 / CHG-0063 Git-metadata-root remediation: Reject every case variant of a `.git` read-root component before opening operation authority |
| 2026-07-26 | v23 / CHG-0063 exact-tree review remediation: Reject non-integer/null request IDs and malformed initialize negotiation, traverse and finally revalidate read-root routes without links/reparse points, reject staged publication after a public-parent identity change, clean every post-link failure's quarantine bytes, and share the transaction root capability |
| 2026-07-27 | CHG-0063-close-independent-mcp-security-review-gaps-for-issue-414: Close independent MCP security review gaps for issue 414 |
| 2026-08-14 | CHG-0121-coverage-over-zero-source-files-must-report-nothing-measured-everywhere-replac: Coverage over zero source files must report nothing measured, everywhere: replace the precomputed percentage fields with Option-returning accessors so no renderer can substitute 100 percent for an unasked question |
| 2026-08-14 | CHG-0123-staleness-that-cannot-be-measured-must-be-refused-not-reported-as-zero-drift-i: Staleness that cannot be measured must be refused, not reported as zero drift, in every reader: report, check --stale, the lifecycle no_stale guard, and the score freshness dimension |
| 2026-08-14 | CHG-0125-every-output-format-must-report-the-same-set-of-findings-so-a-machine-readable: Every output format must report the same set of findings, so a machine-readable consumer cannot see fewer problems than a human reading the text |
