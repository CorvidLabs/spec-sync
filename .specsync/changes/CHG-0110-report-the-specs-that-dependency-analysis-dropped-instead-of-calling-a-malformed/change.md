---
id: CHG-0110-report-the-specs-that-dependency-analysis-dropped-instead-of-calling-a-malformed
state: implementing
type: bug_fix
base_commit: 7bd6c0ac75ecf83bf680a303d3146709021423f1
---

# Report the specs that dependency analysis dropped instead of calling a malformed graph valid

## Intent

Report the specs that dependency analysis dropped instead of calling a malformed graph valid

## Affected Canonical Specs

- `deps`

## Acceptance Criteria

- Running `specsync deps` against a spec whose frontmatter is malformed reports the defect with the same wording `specsync check` uses and exits non-zero, instead of treating the declaration as empty and printing that all dependency declarations are valid. A spec whose frontmatter cannot be parsed at all, and one that declares no module, are each reported as dropped from the analysis rather than skipped silently. A project whose specs are all well-formed continues to report a valid graph and exit zero.

## No-spec Rationale

Not applicable
