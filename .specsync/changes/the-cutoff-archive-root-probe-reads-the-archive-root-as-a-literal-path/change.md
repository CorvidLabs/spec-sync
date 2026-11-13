---
id: the-cutoff-archive-root-probe-reads-the-archive-root-as-a-literal-path
state: implementing
type: bug_fix
base_commit: 18385ef5ead84f0b51d70a29a6fed8a3e5ca222a
---

# The cutoff archive-root probe reads the archive root as a literal path

## Intent

The cutoff archive-root probe reads the archive root as a literal path

## Affected Canonical Specs

- `change`

## Acceptance Criteria

- for a project in a directory named :(exclude)app whose trusted cutoff has no archive root, the cutoff-tree check answers from the cutoff's own tree instead of reporting that the archive tree could not be read; an unreadable cutoff archive tree is still an error

## No-spec Rationale

REQ-change-145 and SPEED-1.a.3 require a cutoff without an archive root to read as absent and only an unreadable archive tree to be an error; a project directory beginning with pathspec magic made the fallback probe fail and report could not be read (#990)
