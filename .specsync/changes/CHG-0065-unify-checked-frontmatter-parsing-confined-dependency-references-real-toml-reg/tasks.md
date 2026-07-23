---
change: CHG-0065-unify-checked-frontmatter-parsing-confined-dependency-references-real-toml-reg
artifact: tasks
---

# Tasks

## Characterization

- [ ] Record failing reproductions for every facet in GitHub issues #413, #419, #422, #436, and
  #444, including exact output and exit status without shell pipelines.
- [ ] Add checked-frontmatter characterizations for duplicate known/unknown keys, scalar/map
  `depends_on`, flow lists, malformed delimiters/quotes/collections/indentation, invalid versions,
  and valid unknown extensions.
- [ ] Add shared-reference characterizations for malformed remote refs, absolute/drive/UNC paths,
  parent traversal, backslashes, symlink escapes, missing bare modules, and duplicate identities.
- [ ] Add registry characterizations for `[specs]`, documented `[[modules]]`, malformed TOML with a
  surviving name, conflicting mappings, inert stubs, and custom mapped spec paths.
- [ ] Add remote characterizations for primary-path fetch, 404-only legacy fallback, auth/no-auth,
  redaction, body limits, shared deadline, preserved timeout/refusal categories, and total-provider-
  failure false green.
- [ ] Add deps/resolve characterizations for inert `--require-coverage`, threshold 101, zero-source
  and manifest-inconclusive gates, local missing exit 0, ignored JSON, doubled heading, and
  sequential blackhole repositories.

## Foundation implementation

- [ ] Add structured frontmatter diagnostics and checked parse API; retain only a delegating legacy
  wrapper for explicitly non-gating compatibility callers.
- [ ] Enforce duplicate, syntax, type, and shape rules while preserving BOM, comments, CRLF,
  deterministic body parsing, and valid unknown extensions.
- [ ] Add the shared `DependencyRef` parser, normalized identity, raw diagnostic spelling, and
  deterministic deduplication.
- [ ] Add confined local resolution with registry-first bare-module mapping, canonical fallback,
  nearest-existing-ancestor checks, and symlink-escape rejection.

## Registry and transport implementation

- [ ] Replace the registry line scanner with typed TOML parsing for `[specs]` and `[[modules]]`;
  fail closed on malformed/conflicting input and keep deterministic `[specs]` emission.
- [ ] Validate and confine every local and remote registry mapping before file, request, or cache
  access; preserve valid inert-stub compatibility.
- [ ] Implement primary `.specsync/registry.toml` fetch with root legacy fallback only on 404.
- [ ] Share authenticated GitHub content transport across registry and spec reads with optional
  `GITHUB_TOKEN`, token redaction, byte limits, bounded concurrency, and one invocation deadline.
- [ ] Make fresh and cached remote content use identical checked parsing and verdict logic.

## Consumer migration

- [ ] Migrate dependency graph construction, validation, cycles, Mermaid, and DOT to checked specs
  and deduplicated typed references; prove typed `implements`/`tracks` remain metadata-only.
- [ ] Wire `deps --require-coverage` through checked coverage in every output mode; reject out-of-
  range thresholds and fail inconclusive gates.
- [ ] Migrate `src/commands/check.rs` to checked frontmatter, shared dependency resolution, and
  conventional versus registry-mapped module identity validation; add focused check integration
  coverage so malformed specs cannot be omitted.
- [ ] Migrate scoring to shared dependency resolution without false penalties or outside-root
  probes.
- [ ] Implement typed `ResolveReport`, structured output, complete-verification success guards,
  bounded repository fetching, and 0/1/2 outcome semantics.
- [ ] Migrate MCP list/detail/graph/check/score paths after #414 integrates, reusing retained-root
  confinement.

## Contracts and evidence

- [ ] Update all affected canonical specs, present companions, public docs, CLI help, and changelog;
  increment versions and remove superseded lenient/line-scanner/unauthenticated contracts.
- [ ] Update sandbox drill `012-registry-parser-realities.sh` and add private candidate drills for
  frontmatter/reference parity (`014`), deps coverage gates (`015`), local resolve/JSON (`016`),
  and authenticated remote registry/spec reads (`017`); do not collide with existing drill `013`.
- [ ] Run targeted unit/integration tests and prove check/deps/resolve/score/MCP diagnostic parity
  across the shared fixture matrix.
- [ ] Run `fledge run fmt`, `fledge run lint`, the complete Rust suite, `fledge lanes run verify`,
  the full repository lane, strict 100% coverage, score >=80, and `fledge trust verify`.
- [ ] Record and verify Attest provenance after all verification gates pass.
- [ ] Complete issue-body acceptance review and a separate adversarial parser/security/compatibility
  review; resolve all high and medium findings.
- [ ] Replay the private sandbox against the exact candidate commit and retain non-secret evidence.
- [ ] Present final evidence for explicit closing approval; use `Fixes #413`, `Fixes #419`,
  `Fixes #422`, `Fixes #436`, and `Fixes #444` only when every facet is satisfied.
