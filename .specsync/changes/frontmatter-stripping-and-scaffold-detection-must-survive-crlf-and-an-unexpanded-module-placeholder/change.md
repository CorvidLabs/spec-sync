---
id: frontmatter-stripping-and-scaffold-detection-must-survive-crlf-and-an-unexpanded-module-placeholder
state: implementing
type: bug_fix
base_commit: 9b6e03cd10d33d1278430b59b8a393d6d672e277
---

# Frontmatter stripping and scaffold detection must survive CRLF and an unexpanded module placeholder

## Intent

Frontmatter stripping and scaffold detection must survive CRLF and an unexpanded module placeholder

## Affected Canonical Specs

- `change`
- `generator`

## Acceptance Criteria

- a pristine generated context companion with CRLF line endings is silent at change new, exactly as the LF one is
- CRLF authored prose is still counted, so suppression and counting agree on both encodings
- the generator hands out the EXPANDED scaffold, so the spec frontmatter line can match a real file instead of an unexpandable placeholder
- frontmatter is stripped at its closing delimiter line in either encoding, and a body horizontal rule still never truncates

## No-spec Rationale

Not applicable
