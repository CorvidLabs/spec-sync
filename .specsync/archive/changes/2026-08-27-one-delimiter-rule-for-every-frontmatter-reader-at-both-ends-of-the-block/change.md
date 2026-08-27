---
id: one-delimiter-rule-for-every-frontmatter-reader-at-both-ends-of-the-block
state: archived
type: bug_fix
base_commit: d6f266a4fd683246469eb15a8f632061dd5cfbb4
---

# One delimiter rule for every frontmatter reader, at both ends of the block

## Intent

one delimiter rule for every frontmatter reader, at both ends of the block

## Affected Canonical Specs

- `parser`
- `change`

## Acceptance Criteria

- A frontmatter delimiter line is three dashes plus trailing whitespace at BOTH ends of the block, in either line encoding, with the two ends free to disagree, and strip_frontmatter, parse_frontmatter and parse_checked_issue_references all apply that one rule; an artifact that is only frontmatter opened with a padded delimiter is refused by the completeness gate instead of approved; prose above the first horizontal rule in a body survives a padded closing delimiter in every reader; a line that is not exactly three dashes — four dashes, an indented three, or three followed by text — is still not a delimiter in any reader; and the two behaviours PR #715 changed without stating them, a BOM-prefixed empty artifact being refused and parse_frontmatter returning an LF body for a CRLF-only body, are pinned by test and written into the specs

## No-spec Rationale

Not applicable
