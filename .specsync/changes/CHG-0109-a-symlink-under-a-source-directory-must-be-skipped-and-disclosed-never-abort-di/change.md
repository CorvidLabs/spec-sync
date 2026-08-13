---
id: CHG-0109-a-symlink-under-a-source-directory-must-be-skipped-and-disclosed-never-abort-di
state: implementing
type: bug_fix
base_commit: 9240b1dfe12e604845b0e083036ce7ea31c2964d
---

# A symlink under a source directory must be skipped and disclosed, never abort discovery

## Intent

A symlink under a source directory must be skipped and disclosed, never abort discovery

## Affected Canonical Specs

- `validator`
- `types`
- `output`
- `cmd_check`
- `commands`
- `comment`
- `generator`
- `cli`

## Acceptance Criteria

- A project containing a symlinked entry under a source directory completes `specsync check` normally — per-spec results, coverage figures, and a summary — instead of aborting with a single discovery error. Every skipped link is named in the text, markdown, and JSON outputs, so the coverage percentages are never read without what was excluded from them. Bare `check` stays exit 0; `--strict` exits 1 naming the exclusion, because a partially-measured tree must not be called clean. A symlink whose target lies outside the project root is still never traversed and its content is never read. A symlink that is itself a configured `source_dirs` entry still fails loudly rather than being silently skipped.

## No-spec Rationale

Not applicable
