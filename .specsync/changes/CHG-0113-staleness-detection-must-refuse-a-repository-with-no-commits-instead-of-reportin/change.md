---
id: CHG-0113-staleness-detection-must-refuse-a-repository-with-no-commits-instead-of-reportin
state: implementing
type: bug_fix
base_commit: 8feb4581d9dcee97028573aab1ee33b00b5cccf4
---

# Staleness detection must refuse a repository with no commits instead of reporting every spec current

## Intent

Staleness detection must refuse a repository with no commits instead of reporting every spec current

## Affected Canonical Specs

- `cmd_stale`
- `git_utils`

## Acceptance Criteria

- Running `specsync stale` in a git repository that has no commits reports that the repository has no commits and exits non-zero, instead of reporting every spec up to date. The same repository, once it has a single commit, reports staleness normally and exits zero. A directory that is not a git repository continues to report that distinctly, so the two reasons are told apart in both the text and machine-readable outputs.

## No-spec Rationale

Not applicable
