---
id: an-excluded-source-file-never-names-a-module-that-coverage-asks-a-spec-for
state: draft
type: bug_fix
base_commit: 18385ef5ead84f0b51d70a29a6fed8a3e5ca222a
---

# An excluded source file never names a module that coverage asks a spec for

## Intent

An excluded source file never names a module that coverage asks a spec for

## Affected Canonical Specs

- `validator`

## Acceptance Criteria

- A flat source file that exclude_patterns excludes names no unspecced module and asks for no spec, while a kept flat file still does (red then green)

## No-spec Rationale

Applies Leif's decision on #613 (orc relay, 2026-11-06: excluded means ignored) and the documented meaning of exclude_patterns (additive source exclusions): the flat-file-stem block now skips an excluded file, as the coverage denominator already did. No requirement text changes.
