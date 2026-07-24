---
spec: validator.spec.md
---

## Tasks

## Post-5.0 Roadmap

- [ ] Add `--fix` mode to auto-fix simple validation errors (e.g., updating stale file paths)
- [ ] Add incremental validation (only re-validate specs whose source files changed)
- [ ] Support `db_tables` validation against ORM model definitions, not just SQL files
- [ ] Add severity levels to warnings (info, warning, error) for more granular CI control

## Done

- [x] Keep coverage regression fixtures warning-free under current stable Clippy
- [x] Bidirectional API surface validation (spec ↔ code exports)
- [x] Frontmatter field validation (module, version, status, files)
- [x] Source file existence checking with Levenshtein suggestions
- [x] Required section presence checking
- [x] Cross-project reference detection and parsing
- [x] File and LOC coverage computation
- [x] Module detection cascade (config → manifest → subdirs → flat files)
- [x] Test file exclusion from coverage
- [x] SQL schema table name extraction
- [x] Exclude pattern matching (glob-style)
- [x] Measure default HTML, HTM, and CSS sources in coverage
- [x] Reject known unfilled companion scaffold markers in strict mode
- [x] Reject every marker emitted by the built-in design companion template
- [x] Add checked coverage and route CLI/MCP gates through inconclusive manifest-discovery errors
- [x] Add crate-visible `validate_spec_content` and route path-based `validate_spec` through the
  shared exact-byte validation core.
- [x] Enable capability-rooted `issues --create` validation to consume immutable pre-read spec
  snapshots without reopening discovered paths.
- [x] Add crate-private `SourceSnapshot` and `validate_spec_content_with_sources` so exact
  spec-and-source validation never falls back to ambient mapped-source reads.

## CHG-0063 Independent-Review Amendment

- [x] Amend checked-coverage contracts so raw Gradle drive identifiers, unsupported
  `setProjectDir` forms, and linked/reparse-backed derived directories remain inconclusive.
- [x] Verify CLI and MCP checked gates return non-success without partial totals, outside bytes, or
  generated output for every new Gradle confinement failure.
- [x] Keep interpolated/encoded Gradle paths and linked/special Gradle manifests inconclusive
  across checked CLI/MCP gates.
- [x] Bind post-manifest coverage source traversal and reads to one retained root capability so
  path replacement, links/reparse points, and special entries fail every coverage gate.
- [x] Replace ambient coverage walks and LOC reopens with retained no-follow source snapshots,
  including deterministic post-discovery Unix symlink and hosted-Windows junction race fixtures.
- [x] Share one retained project capability across manifest discovery, spec-module enumeration,
  configured source traversal, and final root verification.
- [x] Read caller-selected spec ownership frontmatter through that retained capability with the
  same no-follow, non-blocking, identity, UTF-8, depth, and cumulative-input enforcement.
- [x] Replace recursive ambient enumeration with deterministic iterative traversal bounded to
  8 MiB per file, 64 MiB cumulative bytes, 100,000 entries, and 256 components.
- [x] Reject invalid-UTF-8 source names/content, special entries, and directory/file/root identity
  replacement without returning partial coverage totals.
- [ ] Obtain fresh exact-tree full reruns, independent reviews, hosted-Windows runtime,
  repository/CI, trust, and Attest evidence.

## Gaps

- No incremental/cached validation — every run re-validates all specs from scratch
- `db_tables` validation only works with raw SQL `CREATE TABLE` statements
- No auto-fix capability for common errors

## Review Status

Per-module role sign-offs were not collected. Release approval is governed by digest-bound change approvals and required CI; this note is informational and is not a release gate.
