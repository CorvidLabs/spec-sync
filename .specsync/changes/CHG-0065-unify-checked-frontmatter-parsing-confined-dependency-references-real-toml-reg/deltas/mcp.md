## ADDED

### REQUIREMENT REQ-mcp-004

MCP dependency resources and read tools SHALL use the same checked parsing and dependency verdicts
as the CLI without weakening the MCP server-root boundary.

Acceptance Criteria

- Spec listing, spec detail, graph, check, and score paths consume checked frontmatter and typed
  dependency references.
- Malformed or escaping dependencies return a tool or resource error instead of a partial graph,
  inflated score, or successful omission.
- Dependency and registry resolution remains beneath the retained MCP server-root capability; this
  change reuses that confinement boundary rather than introducing ambient path joins.
- MCP structured results preserve raw offending references and deterministic diagnostics without
  exposing outside-root content.

## MODIFIED

### SPEC SECTION Invariants

1. Protocol version is `2024-11-05` and the server reports tools and resources capabilities.
2. Read-only mode exposes five non-mutating tools; write mode additionally exposes
   `specsync_generate` and `specsync_init`.
3. The requested root is opened and identity-bound before canonicalization; the canonical path is
   reopened and must identify the same directory.
4. Read tools validate lexical descendants and open them only through the retained server-root
   capability; ambient root replacement cannot redirect authority.
5. Mutating tools require write mode, reject root overrides, and use the configured root.
6. Complete request envelopes, tool arguments, and resource arguments are validated before
   dispatch; tool-domain errors use `isError` and protocol-shape errors use JSON-RPC errors.
7. Every notification receives no response and cannot mutate state.
8. Config, metadata, cache, manifest, autodetection, dependency, registry, schema, module, spec
   mapping, nested-symlink, and write paths remain confined through the retained root capability.
9. Recursive checks honor ignored and configured exclusions and use deterministic cumulative input
   budgets; bounded snapshots copy the exact bytes charged during discovery.
10. JSON-RPC input lines and responses remain bounded to 1 MiB and request IDs to 4 KiB.
11. Generated output remains bounded, preflighted, staged, synchronized, atomically published
    without overwrite, and rolled back only through retained capabilities and matching
    filesystem-plus-content identities.
12. Spec listing, detail, graph, check, and score paths use checked frontmatter and shared typed
    dependency references; malformed specs or dependencies cannot become partial success.
13. Dependency and registry resolution occurs beneath the retained server-root capability and
    rejects lexical, symlink, nearest-existing-ancestor, and configured-mapping escapes before
    downstream access.
14. MCP tool and resource errors preserve deterministic parser or dependency categories and exact
    raw offending references without exposing outside-root content.
15. Manifest-derived inputs retain the documented real Cargo TOML and shared Gradle parsing and
    explicit-input behavior.
16. GitHub issue verification retains explicit-token, in-process, bounded, deduplicated, and
    fail-closed behavior; provider failures remain inconclusive tool errors.
