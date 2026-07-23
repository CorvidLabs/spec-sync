---
change: CHG-0065-unify-checked-frontmatter-parsing-confined-dependency-references-real-toml-reg
artifact: research
---

# Research

## Issue-to-root-cause map

| Issue | Reproduced contract gap | Current root cause |
|------|--------------------------|--------------------|
| #413 | Documented `[[modules]]` mappings disappear; malformed TOML with `name = {{{` is accepted | `src/registry.rs::scan_registry_fields` is a line scanner that only collects `[specs]`; `parse_registry` accepts any surviving non-empty name |
| #419 | `deps --require-coverage` is inert; scalar `depends_on` vanishes; duplicate declarations inflate diagrams/counts | `src/main.rs` does not pass the global threshold to `cmd_deps`; parser shape is unchecked; graph construction neither diagnoses malformed specs nor deduplicates edges |
| #422 | Remote resolve requests the wrong registry path, cannot authenticate private reads, prints success after total failure, and exits 0 | `fetch_remote_registry` requests root `specsync-registry.toml` through raw GitHub; failures are stored as `None`; an empty drift vector is treated as proof; outcome is printed rather than returned |
| #436 | Duplicate keys can select draft, flow dependencies vanish, unsafe paths disagree across commands, module identity is unchecked, malformed YAML reports valid | `parse_frontmatter` is regex plus last-write line scanning; validator, deps, resolve, score, and MCP each interpret strings independently |
| #444 | Missing local refs and malformed refs exit 0, traversal/absolute refs can validate, JSON is ignored, remote blackholes cost one timeout each | `cmd_resolve` directly joins local strings, malformed remote parsing returns `None`, fetch loops are sequential, and there is no typed report/outcome |

## Current implementation observations

- `parse_frontmatter` returns `Option<ParsedSpec>` and silently ignores malformed lines, unknown
  shape, and duplicate assignment. `set_scalar` and `set_field` make scalar/list interpretation
  dependent on formatting rather than declared field schema.
- `depends_on: [alpha]`, which generated specs use, is not parsed as a dependency list by the
  current scanner. This explains the zero-edge control in #436 and makes a parser fix a prerequisite
  for graph behavior.
- Existing canonical parser and registry companions still promise lenient zero-dependency line
  parsing. Those statements conflict with the five issue requirements and must be explicitly
  revised, not preserved as accidental compatibility.
- `validator.rs` resolves bare dependencies as `specs_dir/<dep>` while `deps.rs` extracts a module
  name and `resolve.rs` performs `root.join(dep).exists()`. These are three different identity and
  existence definitions.
- `validator::source_within_root` already demonstrates nearest-existing-ancestor confinement for
  source mappings. Dependency resolution should share the principle but must additionally resolve
  registry authority and canonical spec identity.
- Registry custom mappings mean `module` cannot always be validated against a parent directory.
  The strict path/name equality rule applies only to conventional
  `specs/<module>/<module>.spec.md`; an explicit valid registry mapping is authoritative.
- `Cargo.toml` already has a direct `toml` dependency used elsewhere. Registry code should consume
  it instead of preserving the zero-dependency scanner.
- `src/github.rs` now contains hardened in-process GitHub REST patterns for explicit token handling,
  deadlines, strict parsing, and redaction. Remote content fetch should reuse those primitives or
  extract a shared transport rather than implementing a second auth/error model.
- The current remote cache stores strings by sanitized repo/path. Cache reads must not bypass path
  validation, response limits, checked frontmatter, or registry parsing.
- MCP #414 adds a retained-root capability boundary. Dependency parity must land after or rebase on
  that work so typed resolution does not reopen ambient root joins.

## Design decisions

1. **One parse, one identity, one verdict.** Checked frontmatter and `DependencyRef` are shared
   infrastructure. Command-specific rendering may differ, but validity may not.
2. **Unknown extensions remain compatible, ambiguity does not.** A unique syntactically valid
   unknown field is allowed. Duplicate unknown fields fail because last-wins ambiguity is itself a
   validation hazard.
3. **Portable local references.** Local references are project-relative slash-separated paths or
   bare modules. Rejecting backslashes and drive/UNC syntax prevents platform-dependent authority.
4. **Registry-first bare resolution.** Explicit registry mappings support non-conventional spec
   locations; conventional lookup is the fallback. Mere directory existence is never sufficient.
5. **Real TOML, canonical emission.** Both existing `[specs]` and documented `[[modules]]` inputs are
   accepted, but SpecSync keeps emitting `[specs]` to avoid output churn and split authority.
6. **404 is absence; other provider failures are inconclusive.** Legacy registry fallback is valid
   only after a real primary-path 404. Authentication, authorization, timeout, rate-limit, malformed
   response, and network failures must not be reinterpreted as absence.
7. **Private and public GitHub reads share one transport.** `GITHUB_TOKEN` is optional for public
   content and required in practice for private repositories; both registry and spec reads attach
   it consistently when present and redact it everywhere.
8. **Bound the invocation, not each repository independently.** Deduplicate repositories and use
   bounded concurrency under one deadline so N blackhole repositories cannot cost N times the
   timeout.
9. **Fail closed without conflating strictness.** Missing, malformed, unsafe, or inconclusive
   dependencies are findings regardless of `--strict`. Strict mode promotes advisory warnings such
   as non-bidirectional references.
10. **Structured output is a contract.** JSON must remain parseable on failure and expose validity,
    gate status, coverage, raw references, normalized identities, and findings; success prose is
    never inferred from an empty error vector after skipped work.

## Characterization matrix

| Fixture | Check | Deps | Resolve | Score | MCP |
|---------|-------|------|---------|-------|-----|
| Duplicate `status`, including hidden `draft` | hard diagnostic | hard diagnostic | hard diagnostic | invalid | tool/resource error |
| `depends_on: alpha` or map shape | hard diagnostic | hard diagnostic in text/JSON/diagram | hard diagnostic | invalid | tool/resource error |
| `depends_on: [alpha, beta]` | two typed refs | two edges | two refs | valid if resolvable | two refs |
| Repeated equivalent dependency | first raw spelling retained | one edge/count/diagram line | one check | one freshness item | one graph edge |
| Missing bare module | error | missing-dependency error | finding, exit 1 | freshness failure | tool/resource error |
| Registry mapping to `custom/lib.spec.md` | valid | one `lib` identity | valid | valid | valid beneath root |
| Absolute/traversal/backslash/symlink escape | same category/error | same category/error | same category/error, exit 1 | no outside probe | tool/resource error |
| `owner/repo@module` | syntax-valid locally | excluded from local edge after syntax check | remotely verifiable | no local-path penalty | syntax-valid and confined |
| `repo@` | malformed with raw text | malformed with raw text | finding, exit 1 | invalid | tool/resource error |
| Malformed registry or remote spec | check fails when local authority is needed | graph fails when authority is needed | finding, exit 1 | invalid if needed | tool/resource error |

## Private sandbox evidence

- Existing `CorvidLabs/spec-sync-sandbox/drills/012-registry-parser-realities.sh` is the primary
  #413 regression. Convert it from a defect reproduction to assertions that `[specs]` and
  `[[modules]]` both resolve custom paths and malformed TOML fails with the established diagnostic.
- Add a frontmatter/dependency parity drill that runs the same fixtures through `check --strict`,
  `deps` text/JSON/Mermaid, `resolve`, and score, asserting exact non-zero statuses and parseable
  JSON without pipelines masking exit codes. Use `014-frontmatter-dependency-parity.sh` because the
  private repository already contains `013-batch-correct-owner.sh`.
- Add a coverage/outcome drill for uncovered sources, threshold 100, invalid threshold 101,
  zero-source, malformed manifest, and complete success controls as
  `015-deps-coverage-gates.sh`.
- Add `016-local-resolve-json.sh` for missing local dependencies, malformed remote-like refs, raw
  diagnostic text, exact exit 1, JSON purity, and the corrected local heading.
- Add a private remote drill using the exact candidate binary and a sandbox-scoped token. It must
  read both `.specsync/registry.toml` and the mapped spec in a private repository, prove primary-path
  behavior, and scan logs/output to ensure the token never appears. Include an anonymous public
  repository control and a deliberately invalid-token failure control. Name it
  `017-private-remote-resolution.sh`.
- Retain command, candidate commit, platform, exit status, and redacted output as review evidence;
  never commit credentials or cache files containing authorization material.

## Risks and mitigations

- **Compatibility:** stricter parsing will expose previously ignored malformed specs. Preserve valid
  extension fields and both list styles, document changed diagnostics, and add release notes.
- **Call-site breadth:** `parse_frontmatter` has many consumers. Add the checked API first, migrate
  gating paths in slices, and keep a temporary delegating wrapper rather than changing every return
  type in one edit.
- **Registry identity:** a naive directory-name check breaks supported custom paths. Carry registry
  authority into module validation and test both conventional and mapped fixtures.
- **Security:** path joins, cache keys, and remote paths are authority boundaries. Validate before
  filesystem/network access and adversarially test symlinks, missing leaves, encoded separators,
  oversized bodies, and credential-bearing errors.
- **False green:** skipped reads must be represented explicitly in reports. Success requires proven
  completion counts, not merely zero collected drift issues.
- **MCP conflicts:** delay `src/mcp.rs` edits until #414 is integrated and have the security reviewer
  confirm retained-root confinement remains intact.
