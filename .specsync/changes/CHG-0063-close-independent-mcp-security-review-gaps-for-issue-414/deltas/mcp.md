## MODIFIED

### REQUIREMENT REQ-mcp-002

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
- Every case variant of `.git` is rejected as a configured input and excluded from snapshots.
- Project inputs are bounded to 8 MiB per file and 64 MiB of actual file/config bytes cumulatively;
  explicitly configured normally ignored roots remain eligible.
- Ignored and configured-exclusion symlink names are skipped before their targets are followed,
  unless an explicit configured input names the path or a descendant; broad ancestor inputs do
  not override configured exclusions.
- Manifest discovery parses Cargo workspace membership as TOML and comment/escape-aware Gradle
  settings, charges deduplicated manifest bytes to the shared cumulative input budget, and copies
  the exact preflight buffers.
- Cargo path discovery follows only semantic target, dependency, workspace-dependency,
  target-specific dependency, patch, and replacement tables; unrelated metadata `path` keys are
  ignored.
- Unix symlink and Windows junction/reparse-point escapes fail before outside access.
- Windows absolute-root components are compared with native ordinal Unicode ignore-case semantics
  without lossy UTF-8 conversion.
- Absolute Windows children may use either original or canonical startup spelling after startup
  identity-binds both spellings; only the relative suffix is opened through the retained canonical
  capability, and sibling-prefix lookalikes are rejected.
- Exact bounded selected-config bytes are validated before compatibility loading; invalid UTF-8,
  malformed JSON/TOML, and wrong-typed specs/source path selectors make every tool/resource
  inconclusive rather than selecting defaults.
- Manifest-relative Cargo paths may normalize `..` across sibling crates when the normalized result
  remains beneath the retained root. Confined Windows-native backslashes normalize equivalently;
  drive, UNC, rooted, traversal, canonical, symlink, and junction escapes remain rejected.

### REQUIREMENT REQ-mcp-003

The MCP server SHALL validate JSON-RPC envelopes and arguments before dispatch and SHALL fail closed
when protocol output or deterministic generation cannot complete safely.

Acceptance Criteria

- Invalid request envelopes return `-32600` before dispatch, including in write-enabled mode.
- Resource arguments are exact-schema validated with `-32602` failures.
- Responses are bounded to 1 MiB; bounded request IDs are preserved while oversized IDs safely fall
  back to `null`, and transport failures are surfaced.
- Generation destination collisions and incomplete writes return tool errors instead of success;
  partial multi-file output is rolled back through retained parent capabilities after filesystem
  identity checks.
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

### SPEC SECTION Invariants

1. Protocol version is "2024-11-05".
2. Server reports tools and resources capabilities.
3. Read-only mode exposes five non-mutating tools; write mode additionally exposes
   `specsync_generate` and `specsync_init`.
4. All filesystem operations remain within the retained canonical server-root capability,
   including read-root selection and generated-file rollback after ambient path replacement.
5. Absolute outside read roots are rejected before filesystem probing; in-root candidates must be
   existing canonical descendants.
6. Mutating tools require write mode, reject root overrides, and use the configured root.
7. Complete JSON-RPC envelopes, tool arguments, and resource arguments are exactly validated before
   dispatch.
8. Tool-domain errors use `isError`; JSON-RPC shape errors use protocol error objects.
9. Every valid notification, including unknown methods, receives no response and cannot mutate.
10. Project-controlled Git metadata is not used for MCP issue-repository discovery and every case
    variant of `.git` is excluded from configuration inputs and snapshots.
11. Project files are bounded to 8 MiB each and actual configured operation inputs, including config
    files, to 64 MiB cumulatively; explicit normally ignored roots remain eligible.
12. JSON-RPC input and output are bounded to 1 MiB; oversized input is drained and oversized output
    becomes a compact `-32603` response with a bounded ID or `null` fallback.
13. Generation collisions and incomplete writes are failures; public transaction paths are
    identity-bound and replacements are preserved. Empty parents created by failed batches may
    remain, and same-user mutation of private transaction names is outside the MCP caller/path
    confinement threat boundary.
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
25. Selected config is validated from exact bounded snapshot bytes before compatibility loading;
    malformed/invalid-UTF-8/wrong-typed path selectors fail tools and resources closed.
