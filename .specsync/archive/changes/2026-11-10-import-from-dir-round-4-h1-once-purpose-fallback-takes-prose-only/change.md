---
id: import-from-dir-round-4-h1-once-purpose-fallback-takes-prose-only
state: archived
type: bug_fix
base_commit: a5b942b2083e405ab7a2379f580e3cf70f33a6da
---

# Import from-dir round 4: H1 once, purpose fallback takes prose only

## Intent

import from-dir round 4: H1 once, purpose fallback takes prose only

## Affected Canonical Specs

- `importer`

## Acceptance Criteria

- the H1 renders exactly once with a blank line after frontmatter; the purpose fallback returns the first prose line of the lead-in, never a numbered list item or fenced code

## No-spec Rationale

code alignment with REQ-importer-002; no contract change
