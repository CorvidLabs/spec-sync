## ADDED

### REQUIREMENT REQ-ignore-001

Ignore rules SHALL combine project and inline suppressions deterministically while limiting each rule to its documented matching scope.

Acceptance Criteria
- `WarningCategory::from_str` parses category names case-insensitively, treating `_` and `-` as equivalent, and accepts short aliases (`requirements`, `stub`, `undocumented`, `schema-mismatch`, `changelog`, `invariants`, `depends-on`); unknown strings return `None`
- `WarningCategory::classify` maps a warning's text to a category, checking the more specific `schema-type-mismatch` before the generic `schema-column`
- `IgnoreRules::load` reads `.specsyncignore` from the project root, skipping blank lines and `#` comments and stripping inline `#` comments; a `category:pattern` line becomes a per-spec rule, a bare `category` becomes a global rule; a missing file yields empty (default) rules, not an error
- `IgnoreRules::parse_inline` extracts categories from `<!-- specsync-ignore: a, b -->` lines in a spec body; a directive missing the closing `-->` is ignored
- `is_suppressed` returns true when the warning's category is suppressed globally, inline, or by a per-spec pattern that the spec's relative path `starts_with`; a warning whose text matches no category (`classify` → `None`) is never suppressed
