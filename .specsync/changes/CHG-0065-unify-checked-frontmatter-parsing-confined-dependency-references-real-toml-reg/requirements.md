---
change: CHG-0065-unify-checked-frontmatter-parsing-confined-dependency-references-real-toml-reg
artifact: requirements
---

# Requirements

This change closes GitHub issues #413, #419, #422, #436, and #444 as one consistency boundary.
Every dependency declaration must receive the same parse, identity, confinement, resolution, and
gate verdict in `check`, `deps`, `resolve`, `score`, and MCP.

## Exact Issue-Facet Binding

| Issue | Definition-bound facets |
|------|--------------------------|
| #413 | Parse real TOML; accept both `[specs]` and documented `[[modules]]`; keep `[specs]` emission; reject malformed TOML with the established local-registry diagnostic. |
| #419 | Enforce `deps --require-coverage` in every renderer, including explicit non-success for 101 and zero-source/inconclusive projects; reject scalar `depends_on`; parse flow/block lists; deduplicate graph edges; intentionally allow valid unknown extension fields; keep typed `implements`/`tracks` issue metadata out of dependency nodes and edges. |
| #422 | Fetch `.specsync/registry.toml` first and use root fallback only after a confirmed 404; attach and redact `GITHUB_TOKEN`; treat zero successful fetches as failure; exit 1 on findings/inconclusive work; honor JSON. |
| #436 | Reject duplicate keys; validate frontmatter syntax, types, shapes, flow/block dependencies, path confinement, dependency existence, and conventional/registry-backed module identity consistently. |
| #444 | Make unresolved and malformed local refs exit 1; retain exact raw text in diagnostics; honor JSON; bound remote fanout under one deadline; remove the doubled local heading. |

### REQ-parser-002

The parser SHALL provide checked frontmatter parsing that rejects ambiguous or malformed metadata
with deterministic, field-specific diagnostics while retaining a compatibility wrapper for
non-gating callers during migration.

Acceptance Criteria

- A checked API returns either a parsed spec or an ordered collection of diagnostics containing a
  diagnostic kind, message, optional field name, and source line when known.
- Missing/unterminated frontmatter delimiters, invalid YAML syntax, colon-less content, malformed
  quotes or flow collections, invalid indentation/tabs, and unsupported known-field shapes produce
  diagnostics instead of a partially populated `Frontmatter`.
- Duplicate top-level keys are rejected before any value can override an earlier value. This
  applies to known fields and syntactically valid unknown extension fields, so a second `status`
  cannot silently select draft behavior.
- `module`, `status`, and `agent_policy` accept only scalar strings; `files`, `db_tables`,
  `depends_on`, and `lifecycle_log` accept only string sequences; `implements` and `tracks` accept
  only sequences of non-negative integer issue numbers.
- `version` accepts the existing generated numeric representation and valid non-empty textual
  version representation, but rejects booleans, maps, sequences, nulls, and non-version text.
- Block and flow sequences have identical semantics. In particular, `depends_on: [alpha, beta]`
  produces two declarations and `depends_on: alpha` or `depends_on: {alpha: beta}` fails.
- Unknown top-level extension fields remain allowed when they are syntactically valid and unique;
  their presence does not alter known fields.
- Leading UTF-8 BOM handling, CRLF normalization, inline comments, deterministic order, and the
  existing Markdown body and Public API extraction behavior remain compatible.
- Gating consumers do not use `Option`-based parsing to silently skip malformed specs. The legacy
  wrapper may remain only for explicitly non-gating compatibility paths and delegates to the
  checked implementation.

### REQ-types-004

SpecSync SHALL represent every `depends_on` value with one shared typed dependency-reference model
that preserves the original declaration for diagnostics and exposes a normalized identity for
deduplication.

Acceptance Criteria

- The model distinguishes a bare local module, a project-relative local spec path, and a remote
  `owner/repository@module` reference.
- Empty values, missing remote owners/repositories/modules (including `repo@`), absolute paths,
  drive/UNC paths, backslash-separated paths, `..` traversal, invalid spec paths, and invalid
  module/repository identifiers are rejected with the original text included in the diagnostic.
- Local path resolution is rooted at the canonical project root and rejects lexical escapes,
  symlink escapes, and missing-leaf paths whose nearest existing ancestor escapes.
- Bare modules resolve through an explicit local registry mapping first, then the canonical
  `specs/<module>/<module>.spec.md` location. A same-named directory without a spec is not success.
- Equivalent repeated declarations are deduplicated by normalized identity while preserving the
  first declaration order and raw spelling for output.
- All check, dependency graph, resolution, scoring, and MCP consumers use this parser and resolver;
  no command retains an independent `contains('@')`, `Path::join`, or filename-stem interpretation.

### REQ-registry-003

Local and remote registries SHALL be parsed as TOML with explicit schema validation and confined
module mappings.

Acceptance Criteria

- The parser accepts the canonical `[registry]` plus `[specs]` mapping emitted by SpecSync.
- The parser also accepts the documented `[registry]` plus `[[modules]]` array-of-tables shape,
  where every table has a string `name` and string `spec` path.
- Generation, initialization, and registration continue to emit one deterministic `[specs]` table
  sorted by module name; public documentation identifies `[specs]` as canonical and `[[modules]]`
  as an accepted compatibility input.
- Malformed TOML, wrong field types, incomplete module entries, duplicate module identities,
  conflicting mappings across accepted shapes, and non-string mappings fail closed. No surviving
  `name =` line can make malformed TOML valid.
- Every mapped spec path is a portable project-relative path confined beneath the project root;
  absolute, traversal, backslash, drive/UNC, and symlink-escaping mappings fail before file access.
- A valid legacy stub with no registry name and no authoritative mappings remains `Ok(None)`.
  Malformed TOML is never classified as inert.
- The established local failure includes `failed to parse local registry <path>` and preserves the
  surrounding resolution context needed to identify which dependency triggered loading.
- Registry-backed non-conventional locations such as `custom/lib.spec.md` resolve successfully and
  are not subjected to conventional directory-name identity rules.

### REQ-github-005

Remote registry and spec retrieval SHALL use one authenticated, bounded GitHub content transport
that distinguishes absence from authentication, provider, and network failures.

Acceptance Criteria

- Remote resolution requests `.specsync/registry.toml` first. It requests legacy root
  `specsync-registry.toml` only when the primary path returns a confirmed 404.
- A 401, 403, timeout, malformed response, body-limit violation, rate limit, or transport failure
  does not trigger legacy fallback and remains an inconclusive error.
- When `GITHUB_TOKEN` is set, registry and spec-content requests both send it as a GitHub API bearer
  token, enabling private-repository verification. Public repositories remain readable without a
  token where GitHub permits anonymous content access.
- Tokens and authorization headers are redacted from every error, debug, cache, and structured
  output path.
- Registry and spec bodies have deterministic byte limits; repository fetches use a bounded
  concurrency limit and one invocation deadline rather than a sequential timeout per repository.
- Timeout, refusal, authentication, authorization, rate-limit, malformed-response, and body-limit
  failures retain their original transport category through text, JSON, cache, and aggregate
  reporting; a deadline expiry is never rewritten as a connection refusal.
- Registry-provided spec paths are validated before constructing a request or cache path.
- A successfully fetched registry is parsed by the same real TOML parser as a local registry, and a
  successfully fetched spec is parsed by the checked frontmatter parser.
- Cache hits and misses have equivalent authentication, parsing, validation, and final verdict
  semantics; cached malformed or unsafe content cannot become a success.

### REQ-validator-009

`specsync check` SHALL fail closed on checked-frontmatter, dependency-reference, registry, module
identity, and confinement errors.

Acceptance Criteria

- Every checked-frontmatter diagnostic is surfaced against the originating spec and is an error in
  strict validation; malformed specs cannot disappear from the result set.
- Every local dependency is resolved through the shared typed resolver. Missing, malformed,
  absolute, traversing, and symlink-escaping references are errors with the offending text.
- Every remote dependency is syntactically validated during local check even when network
  verification is not requested.
- For a conventional `specs/<module>/<module>.spec.md` path, frontmatter `module` must equal the
  canonical module identity. Registry-mapped custom paths use the registry mapping as authority and
  are not rejected merely because their parent directory differs.
- Check, deps, resolve, score, and MCP fixtures assert identical normalized dependency identities
  and compatible diagnostics for the same malformed or unresolved reference.

### REQ-deps-002

Dependency graph construction SHALL consume checked specs and typed references without silently
dropping declarations or inflating graph results.

Acceptance Criteria

- A spec read or frontmatter parse failure is retained as a hard graph error rather than omitted.
- Flow and block `depends_on` lists produce the same graph edges.
- Repeated equivalent local declarations produce one edge, one edge count, and one Mermaid/DOT
  relationship.
- Bare modules, canonical spec paths, and registry-mapped custom spec paths resolve to one module
  identity. Remote references are syntax-checked but excluded from the local graph.
- Valid `implements` and `tracks` issue-number lists remain typed frontmatter metadata only. They
  are retained for validation and downstream issue features but intentionally create no dependency
  node, edge, cycle, count, Mermaid relationship, or DOT relationship.
- Malformed, unsafe, missing, or identity-conflicting dependencies produce the same failures in
  validation, JSON, Mermaid, and DOT modes.
- Graph node and edge ordering remains deterministic.

### REQ-cmd-deps-002

`specsync deps --require-coverage <percent>` SHALL enforce the checked project coverage gate in
every output mode.

Acceptance Criteria

- The dispatcher passes the global coverage threshold to `cmd_deps`; it is not ignored.
- Text, JSON, Markdown/GitHub, Mermaid, and DOT modes compute coverage through the same checked
  coverage API and return exit 1 when measured coverage is below the requested threshold.
- Zero discoverable source files, malformed manifests, unreadable configured inputs, or another
  inconclusive coverage result fail the requested gate instead of reporting 100% or success.
- Thresholds outside 0 through 100, including 101, are rejected as usage errors with exit 2.
- Therefore 101 is explicitly non-success and can never be ignored or reported as a satisfied
  coverage gate.
- JSON remains parseable on every outcome and includes `valid`, `gate_passed`, the requested
  threshold, project coverage, graph counts, diagnostics, and deduplicated edges.
- Diagram output remains the only stdout payload in Mermaid/DOT mode; gate diagnostics use stderr
  and the exit status without corrupting the diagram.

### REQ-scoring-002

Spec quality scoring SHALL use checked frontmatter and typed dependency resolution without probing
outside the project or penalizing valid non-path references.

Acceptance Criteria

- Malformed frontmatter or dependency references produce an invalid/zero-gating score with an
  actionable, non-secret diagnostic; they do not receive freshness credit.
- Valid bare modules, registry-mapped paths, and remote references are not treated as missing
  project-root paths.
- Missing or unsafe local dependencies receive the same normalized finding as check/deps/resolve.
- Scoring never opens an absolute, traversing, or symlink-escaping dependency target and does not
  leak outside-root identifiers in suggestions or MCP output.

### REQ-cmd-resolve-002

`specsync resolve` SHALL return one explicit report and trustworthy exit status for local and remote
dependency verification.

Acceptance Criteria

- Local missing, malformed, unsafe, registry-load, and module-identity failures are findings and
  exit 1 with or without `--strict`.
- `--remote` treats registry fetch failure, absent registry, missing remote module, malformed
  registry, and unsafe registry mapping as inconclusive/failing findings and exits 1.
- `--verify` treats remote spec fetch/parse failure and breaking drift as findings and exits 1.
  Warning-only compatibility findings such as a non-bidirectional reference exit 0 by default and
  exit 1 under `--strict`.
- The command prints “verified — no drift detected” only when at least one remote registry was
  successfully fetched and every requested remote reference and spec content required by the mode
  was verified.
- `--verify` continues to imply `--remote`; no network request occurs without either flag.
- Text output uses a single “Local dependencies” heading and never drops malformed references.
- JSON is honored and contains `valid`, `gate_passed`, mode, checked counts, cache status, and every
  finding with originating spec, raw reference, normalized identity when available, category,
  severity, and message. No ANSI or human preamble contaminates JSON.
- Remote transport findings preserve their category in every renderer and cache state, including
  reporting a bounded deadline expiry as `timeout` rather than `connection_refused`.
- Trustworthy/advisory success exits 0, findings or inconclusive gates exit 1, and CLI misuse exits
  2.

### REQ-cli-006

CLI dispatch SHALL carry dependency coverage, output format, remote depth, and strictness into the
selected command without accepting an inert flag.

Acceptance Criteria

- The global `--require-coverage` value reaches `deps` in text, structured, Mermaid, and DOT modes.
- The selected output format reaches `resolve`; `--json` cannot be parsed and then ignored.
- `--verify` continues to imply `--remote`, while local mode remains network-free.
- Help and public documentation state that runtime findings/inconclusive work exit 1 and CLI usage
  errors, including an out-of-range coverage percentage, exit 2.

### REQ-commands-003

Shared command outcomes SHALL distinguish trustworthy completion, findings, inconclusive gates,
and usage errors before rendering or exiting.

Acceptance Criteria

- Trustworthy/advisory success maps to exit 0, findings and inconclusive gates map to exit 1, and
  usage errors map to exit 2.
- JSON serializes the complete result before exit and is the only stdout payload in JSON mode,
  including parser, registry, coverage, fetch, and resolution failures.
- Check, deps, resolve, scoring, and MCP integration fixtures assert the same dependency category,
  normalized identity when available, and exact raw spelling for the same declaration.

### REQ-mcp-004

MCP dependency resources and read tools SHALL use the same checked parsing and dependency verdicts
as the CLI without weakening the MCP server-root boundary.

Acceptance Criteria

- Spec listing, spec detail, graph, check, and score paths consume checked frontmatter and typed
  dependency references.
- Malformed or escaping dependencies return a tool/resource error instead of a partial graph,
  inflated score, or successful omission.
- Dependency and registry resolution remains beneath the retained MCP server-root capability; this
  change reuses the #414 confinement boundary rather than introducing ambient path joins.
- MCP structured results preserve raw offending references and deterministic diagnostics without
  exposing outside-root content.

### Verification and closure

- Each issue facet has a failing characterization committed before its implementation fix.
- Canonical specs and present companions for parser, types, registry, deps, cmd-deps, validator,
  scoring, cmd-resolve, GitHub transport, CLI dispatch, and MCP are updated with version and change
  log increments where their contract changes.
- `CorvidLabs/spec-sync-sandbox` contains candidate-binary drills for real registry parsing,
  frontmatter/dependency parity, coverage/exit gates, and authenticated private remote verification.
- Targeted tests, the full Rust suite, `fledge lanes run verify`, the full repository lane, strict
  100% spec coverage, score at least 80, and `fledge trust verify` pass.
- An implementation review checks every issue-body facet, and an independent adversarial review
  checks parser ambiguity, filesystem confinement, authentication redaction, bounded network work,
  cache parity, and false-green output. All high and medium findings are resolved.
