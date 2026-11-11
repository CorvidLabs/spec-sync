---
id: indented-fence-markers-are-content-in-spec-scans
state: archived
type: bug_fix
base_commit: 46f55652649a72ddf5e1715938b504b96686566b
---

# Indented fence markers are content in spec scans

## Intent

indented fence markers are content in spec scans

## Affected Canonical Specs

- `cmd_check`
- `parser`

## Acceptance Criteria

- a spec with a 4-space-indented fence marker before its export table has its exports read and --fix adds each undocumented export exactly once to the real table

## No-spec Rationale

REQ-cmd-check-017 and REQ-parser-004 already require CommonMark content treatment of indented markers; the scanners disagreed
