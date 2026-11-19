---
id: the-delta-materializer-skips-fenced-examples-when-it-matches-a-requirement-block
state: implementing
type: bug_fix
base_commit: 18385ef5ead84f0b51d70a29a6fed8a3e5ca222a
---

# The delta materializer skips fenced examples when it matches a requirement block

## Intent

The delta materializer skips fenced examples when it matches a requirement block

## Affected Canonical Specs

- `change`

## Acceptance Criteria

- A MODIFIED delta for a requirement whose header also appears in a fenced example above it replaces the real block and keeps the example byte for byte; a heading inside a fenced example within the real block does not end the block (red then green)

## No-spec Rationale

Makes the materializer match REQ-change-152, which has it match a block by the shared requirement-heading reader; that reader treats a fenced example as quoted text, never a requirement. The change spec only lists the new test file.
