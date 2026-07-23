---
change: CHG-0065-unify-checked-frontmatter-parsing-confined-dependency-references-real-toml-reg
artifact: plan
---

# Plan

## Delivery strategy

Implement this cluster in dependency order within one PR. The checked parser and typed reference
model are foundations; registry, graph, validation, scoring, resolve, and MCP must not each invent
their own compatibility interpretation. Capture a failing characterization for every issue facet
before changing behavior.

## Phase 1 — checked frontmatter and typed references

1. Add structured frontmatter diagnostic types and a checked parse result in `src/types.rs` and
   `src/parser.rs`. Replace the permissive last-wins scanner with a checked parser for SpecSync's
   supported YAML subset, retaining `parse_frontmatter` only as a documented compatibility wrapper
   while gating call sites migrate.
2. Validate duplicate known and unknown keys, scalar/list shapes, issue-number sequences, version
   values, delimiters, quotes, flow collections, indentation, and malformed lines. Preserve valid
   unknown extension fields, BOM behavior, comments, CRLF normalization, and Markdown body bytes.
3. Add `DependencyRef` and typed error/report structures. Parse bare modules, portable local spec
   paths, and `owner/repository@module`; preserve raw text; normalize identities; reject malformed,
   absolute, drive/UNC, backslash, traversal, and invalid references.
4. Add one root-confined local resolver. Resolve registry mappings before conventional spec paths,
   canonicalize existing targets or the nearest existing ancestor, reject symlink escapes, and
   deduplicate normalized identities in declaration order.
5. Add parser/type unit tests for duplicate `status`, duplicate extension keys, scalar/map
   `depends_on`, valid flow and block lists, malformed syntax, invalid versions, unknown valid
   extensions, malformed remote refs, traversal/absolute/Windows paths, symlink escapes, and stable
   deduplication.

## Phase 2 — real registries and bounded authenticated transport

6. Replace `scan_registry_fields` with typed TOML deserialization in `src/registry.rs`. Accept
   `[specs]` and `[[modules]]`, merge only non-conflicting identities, validate every field and path,
   preserve the valid inert-stub exception, and keep `[specs]` as deterministic emitted output.
7. Route registry parsing and registration through one typed representation so load, generate,
   register, local dependency resolution, and remote resolution cannot disagree. Preserve the
   established `failed to parse local registry` diagnostic context.
8. Build one GitHub content transport shared by registry and spec fetches, reusing the hardened
   request/deadline/redaction patterns in `src/github.rs` where practical. Request
   `.specsync/registry.toml` first and fall back to root `specsync-registry.toml` only on a real 404.
9. Attach optional `GITHUB_TOKEN` authorization to both registry and spec requests; support
   anonymous public reads; redact credentials; cap response bytes, concurrency, and the whole
   invocation deadline; validate paths before requests and cache writes.
10. Add unit tests with a fake transport for primary-path success, 404-only fallback, no fallback on
    401/403/network failures, token present/absent behavior, redaction, oversized responses,
    malformed TOML, conflicting mappings, custom mapped paths, transport-category preservation,
    and cache hit/miss parity.

## Phase 3 — graph, validation, coverage, and scoring parity

11. Migrate `src/deps.rs` to checked specs and typed references. Retain malformed/unreadable specs
    as graph errors; normalize bare/path/custom-registry identities; deduplicate edges before
    counts, cycle detection, Mermaid, and DOT rendering.
12. Extend `DepsReport` with validity, normalized diagnostics, coverage result, threshold, and gate
    status. Pass `--require-coverage` from `src/main.rs` into `src/commands/deps.rs`; use
    `compute_coverage_checked` in every output mode. Reject thresholds outside 0..=100 as usage
    errors and fail inconclusive coverage gates.
13. Migrate `src/validator.rs` and `src/commands/check.rs` to checked parsing and the shared
    resolver. Enforce conventional module identity without breaking custom registry mappings.
    Validate remote syntax locally and make malformed/missing/unsafe dependencies hard errors;
    add focused `tests/integration/check.rs` coverage so malformed specs cannot disappear.
14. Migrate `src/scoring.rs` to checked parsing and typed resolution. Valid bare, registry, and
    remote references receive no false missing-path penalty; malformed/unsafe refs fail closed
    without probing or disclosing outside-root bytes.
15. Add one fixture matrix exercised through check, deps text/JSON/diagram, and score to prove
    flow-list edges, scalar rejection, duplicate deduplication, missing bare modules, custom
    registry mappings, module/path mismatches, absolute/traversal/symlink rejection, remote syntax,
    and warm/cold parity.

## Phase 4 — resolve report and CLI outcomes

16. Replace tuple accumulation and direct `process::exit` branches in `src/commands/resolve.rs` with
    a typed `ResolveReport` and a shared command outcome returned to the dispatcher.
17. Make every local parse/resolution failure and every remote registry/spec inconclusive result a
    finding. Reserve warning-only status for advisory compatibility results and let `--strict`
    promote those warnings.
18. Honor structured output, include every raw reference and normalized finding, remove the doubled
    heading, and guard success text on complete verification rather than an empty issue vector.
19. Fetch unique repositories with bounded concurrency under one invocation deadline. Preserve
    `--verify` implying `--remote`, avoid network in local mode, and ensure cache contents are parsed
    and validated exactly like fresh content.
20. Wire command outcomes in `src/cli.rs`, `src/main.rs`, and `src/commands/mod.rs`: exit 0 for
    trustworthy/advisory success, 1 for findings or inconclusive gates, and 2 for usage errors.

## Phase 5 — MCP, contracts, and private testbed

21. After the #414 MCP security branch is integrated or rebased, migrate MCP spec-list/detail,
    graph, check, and score paths to checked parsing and typed references. Reuse retained-root
    confinement and return tool/resource errors for malformed or escaping dependencies.
22. Update canonical specs and all present companions for parser, types, registry, deps, cmd-deps,
    validator, scoring, cmd-resolve, GitHub transport, CLI/root dispatch, and MCP. Remove obsolete
    promises of line-scanned TOML/YAML, unauthenticated raw-only fetches, and warning-only unresolved
    dependencies. Update `site/src/content/docs/cross-project-refs.md`, `cli.md`, and
    `spec-format.md`, plus `CHANGELOG.md`.
23. In private `CorvidLabs/spec-sync-sandbox`, update `drills/012-registry-parser-realities.sh` to
    assert both accepted registry shapes and malformed-TOML failure. Add candidate-binary drills:
    `014-frontmatter-dependency-parity.sh`, `015-deps-coverage-gates.sh`,
    `016-local-resolve-json.sh`, and `017-private-remote-resolution.sh`. These names follow the
    existing `013-batch-correct-owner.sh` without collision. The private drill must verify
    authenticated registry and spec reads without printing its token; a public control must verify
    anonymous access.
24. Run targeted parser/registry/deps/resolve/scoring/MCP tests, all integration tests, format and
    lint, strict checks, 100% coverage, score >=80, repository/verify lanes, and trust verification.
    Record Attest provenance only after verification passes.
25. Use separate implementation and reviewer agents. Reviewer one maps tests and evidence to every
    acceptance row in #413/#419/#422/#436/#444; reviewer two performs adversarial parser,
    confinement, auth/redaction, network-bound, cache-parity, and false-green review. Resolve every
    high and medium finding before requesting closing approval.

## Merge-conflict boundaries

- `src/parser.rs`, `src/types.rs`, and their specs are the serial foundation and have one owner.
- `src/registry.rs`/`src/github.rs` and `src/deps.rs`/`src/commands/deps.rs` may proceed in parallel
  after Phase 1, with distinct owners.
- `src/validator.rs` and `src/scoring.rs` may proceed in parallel after the shared resolver lands.
- `src/cli.rs`, `src/main.rs`, `src/commands/mod.rs`, `src/commands/check.rs`, check/command
  integration fixtures, docs, and `CHANGELOG.md` have one final integration owner.
- `src/mcp.rs` is held until #414 is integrated; no cluster agent independently rewrites its
  confinement helpers.
