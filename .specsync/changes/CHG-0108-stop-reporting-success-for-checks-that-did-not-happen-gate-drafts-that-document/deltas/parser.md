## ADDED

### REQUIREMENT REQ-parser-002

Frontmatter parsing SHALL resolve a single- or double-quoted block list item or scalar to
the text inside the quotes, for every field.

Acceptance Criteria
- Quoted entries in `files:`, `depends_on:`, and `db_tables:` resolve to the path inside the quotes, for single and double quotes, mixed with unquoted entries in the same list.
- Quoted scalars such as `module:` and `status:` resolve to the text inside the quotes.
- A comment following the closing quote is discarded; a `#` inside the quotes is retained as content.
- An opening quote with no matching close is a frontmatter error naming the offending value, and the value is not retained as a literal.
- Flow-style lists continue to unquote their own items.
