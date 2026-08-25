---
id: one-canonical-frontmatter-reader-for-crlf-checkouts
state: verifying
type: bug_fix
base_commit: e82542d19ce8d79926b144a0e38d4d620b120715
---

# One canonical frontmatter reader for CRLF checkouts

## Intent

One canonical frontmatter reader for CRLF checkouts

## Affected Canonical Specs

- `parser`
- `view`
- `change`

## Acceptance Criteria

- specsync view renders a CRLF spec instead of failing with 'Cannot parse frontmatter'; parse_frontmatter accepts CRLF input and always returns an LF-only body while an LF document still allocates nothing; a single parser::strip_frontmatter is correct on LF, CRLF, a leading BOM, unterminated frontmatter, a closer at EOF and a body horizontal rule, and view::strip_frontmatter plus change::strip_yaml_frontmatter are deleted rather than left in parallel; a CRLF change artifact with a written body is no longer refused as incomplete and a frontmatter-only artifact closed at EOF is; .specsync/**/*.md is pinned to eol=lf beside the existing JSON pin; cargo fmt, cargo clippy -D warnings and the full test suite pass.

## No-spec Rationale

Not applicable
