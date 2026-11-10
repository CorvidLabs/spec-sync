---
id: import-from-dir-preserves-source-content
state: implementing
type: bug_fix
base_commit: 49d2326a8f80232b8fee94a5073c4c1a66fd2057
---

# Import --from-dir preserves source content

## Intent

import --from-dir preserves source content

## Affected Canonical Specs

- `importer`
- `cmd_import`

## Acceptance Criteria

- import --from-dir keeps the source file's module name, files list, purpose and existing sections (Public API, Invariants, Change Log); the imported spec passes specsync check --strict immediately; the Change Log names the real source instead of Confluence; a source with no usable files list is refused loudly and counted as an error

## No-spec Rationale

Not applicable
