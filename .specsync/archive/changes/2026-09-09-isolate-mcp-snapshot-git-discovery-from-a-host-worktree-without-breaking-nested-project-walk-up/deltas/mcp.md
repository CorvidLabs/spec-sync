## MODIFIED

### REQUIREMENT REQ-mcp-008

MCP scoring and any other MCP path that probes git history SHALL spawn `git` through `git_utils`, inheriting the sanitized child environment rather than the parent process environment. Tool and resource dispatch on a snapshot SHALL pin `GIT_CEILING_DIRECTORIES` to the snapshot parent via `git_utils::with_discovery_ceiling` for the duration of those probes. CLI `git_cmd` callers SHALL NOT pin a ceiling.

Acceptance Criteria
- A read-only `specsync mcp` process that holds `GITHUB_TOKEN` for issue verification does not forward that token to `git`.
- An MCP snapshot sitting inside a host worktree cannot walk up into that worktree via `GIT_DIR` walk-up.
- Snapshot scoring reports `git_freshness_available` false and withholds git freshness once — it does not double-penalize when the tempfile happens to sit inside a host worktree.
- Test fixtures constructing `GenerationOutcome` populate `skipped_no_files`.

### SPEC SECTION Invariants

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
14. Snapshot scoring reports Git freshness unavailable and withholds freshness credit. Dispatch pins `GIT_CEILING_DIRECTORIES` to the snapshot parent so a tempfile inside a host worktree cannot discover host history or double-penalize freshness.
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

### SPEC SECTION Dependencies

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
| git_utils | `with_discovery_ceiling` so snapshot git probes cannot walk up into a host worktree |

### Consumed By

| Module | What is used |
|--------|-------------|
| main | `run_mcp_server` (via `mcp` subcommand) |
