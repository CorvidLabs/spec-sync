---
id: check-reports-a-module-s-3md-lesson-file-that-does-not-parse
state: implementing
type: feature
base_commit: 18385ef5ead84f0b51d70a29a6fed8a3e5ca222a
---

# Check reports a module's .3md lesson file that does not parse

## Intent

check reports a module's .3md lesson file that does not parse

## Affected Canonical Specs

- `validator`
- `cmd_check`

## Acceptance Criteria

- A spec whose <module>.3md does not parse gets one check warning naming the file, the parser's error and context.md, and check --strict fails; a .3md that parses, or none, adds no finding (red then green)

## No-spec Rationale

Not applicable
