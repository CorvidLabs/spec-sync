---
id: import-from-dir-review-round-3-no-h1-docs-keep-sections-purpose-fallback-never-takes-structure-long-fences
state: implementing
type: bug_fix
base_commit: 63319e1bd96c3bdd9e1049d3298b42ed04825f89
---

# Import --from-dir review round 3: no-H1 docs keep sections, purpose fallback never takes structure, long fences

## Intent

import --from-dir review round 3: no-H1 docs keep sections, purpose fallback never takes structure, long fences

## Affected Canonical Specs

- `importer`

## Acceptance Criteria

- a markdown source with no H1 keeps its sections and lead paragraphs; the purpose fallback never emits a heading, table row, fence or list marker; four-backtick fences nest three-backtick fences without splitting sections

## No-spec Rationale

code alignment with already-materialized REQ-importer-002; no contract change
