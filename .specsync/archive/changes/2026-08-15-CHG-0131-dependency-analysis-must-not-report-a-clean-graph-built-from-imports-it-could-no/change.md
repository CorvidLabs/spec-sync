---
id: CHG-0131-dependency-analysis-must-not-report-a-clean-graph-built-from-imports-it-could-no
state: archived
type: bug_fix
base_commit: fb763cc40f08502e6efa2554d4c2bbd1943a62af
---

# Dependency analysis must not report a clean graph built from imports it could not read or could not resolve, so an unattributable import is disclosed rather than dropped

## Intent

Dependency analysis must not report a clean graph built from imports it could not read or could not resolve, so an unattributable import is disclosed rather than dropped

## Affected Canonical Specs

- `deps`
- `cmd_deps`

## Acceptance Criteria

- Kotlin imports are collected and resolved against a package topology built first from each JVM file's own package declaration and then from directory layout, so a file whose directory does not match its package still produces an edge. Every imported package resolves to exactly one of three outcomes: owned by a spec module, foreign to the project's namespace, or inside the project's namespace but unattributed. The third is recorded and disclosed in every output format rather than dropped. A language that has an import construct but no extractor is disclosed; a language with no import concept at all is not. Neither disclosure is an error or a warning, and neither changes the exit code.

## No-spec Rationale

Not applicable
