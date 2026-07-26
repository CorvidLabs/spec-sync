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
- [x] #421: distinguish explicit empty draft file lists from YAML null
- [x] #421: keep freshly generated draft markers advisory-free until promotion

## Gaps

- No incremental/cached validation — every run re-validates all specs from scratch
- `db_tables` validation only works with raw SQL `CREATE TABLE` statements
- No auto-fix capability for common errors

## Review Status

Per-module role sign-offs were not collected. Release approval is governed by digest-bound change approvals and required CI; this note is informational and is not a release gate.
