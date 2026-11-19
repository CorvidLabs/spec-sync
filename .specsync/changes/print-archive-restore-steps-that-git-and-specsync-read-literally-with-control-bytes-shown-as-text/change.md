---
id: print-archive-restore-steps-that-git-and-specsync-read-literally-with-control-bytes-shown-as-text
state: implementing
type: bug_fix
base_commit: 247b6d2a62b93c7f24cbafb56c93e754f9edf85c
---

# Print archive restore steps that Git and SpecSync read literally, with control bytes shown as text

## Intent

Print archive restore steps that Git and SpecSync read literally, with control bytes shown as text

## Affected Canonical Specs

- `change`

## Acceptance Criteria

- the printed corrupt-archive restore puts back only the named package when its path holds * or ?, and the printed archiving-commit lookup and the probes read it literally; every printed specsync change status command reads an id that begins with - as the id; a package path holding a newline or escape byte is printed with no control byte and, run through bash, still restores that exact directory

## No-spec Rationale

Not applicable
