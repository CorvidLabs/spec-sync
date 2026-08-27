## ADDED

### REQUIREMENT REQ-parser-003

Every frontmatter reader in this module SHALL recognize a delimiter LINE by one rule — exactly
three dashes followed by nothing but whitespace — applied to BOTH ends of the block, in either
line encoding, with the two ends free to disagree with each other.

Acceptance Criteria
- A delimiter carrying trailing spaces or tabs opens and closes frontmatter in `strip_frontmatter`,
  `parse_frontmatter`, and `parse_checked_issue_references` alike, so a document with a padded
  OPENER has its YAML removed from the body rather than counted as prose, and a document with a
  padded CLOSER keeps the body prose above the first horizontal rule below it.
- A padded closer never lets frontmatter run into the body: `parse_frontmatter` emits no
  "Ignoring malformed frontmatter line" warning for body prose, and `parse_checked_issue_references`
  reads the references that are there instead of reporting the YAML invalid.
- `parse_checked_issue_references` reads a document whose opening and closing delimiters carry
  different line endings, which the hand-rolled pair of prefix/split chains it replaces could not.
- A line that is not exactly three dashes is not a delimiter in any reader: `----`, `--- x`,
  `---change: x`, and an indented `  ---` leave `strip_frontmatter` returning the document whole,
  `parse_frontmatter` returning `None`, and `parse_checked_issue_references` returning its stable
  content-free error. Loosening this would cut the body of any document that opens with a Markdown
  thematic break at its next rule.
- The three readers return the same verdict for every delimiter shape, asserted as a matrix, so the
  rule cannot be loosened in one reader and not the others.
- `parse_checked_issue_references` keeps the verdicts it had for an empty frontmatter block and for
  a block that is a single blank line.
- `parse_frontmatter` returns an LF-only body when the frontmatter is LF and only the body is CRLF.

## MODIFIED

### SPEC SECTION Invariants

1. `parse_frontmatter` returns `None` unless the content opens on a `---` delimiter line and closes on a later `---` delimiter line; both LF and CRLF encodings are accepted, and a leading UTF-8 BOM never hides the opening delimiter
2. `get_spec_symbols` only extracts the complete first nonempty backtick-quoted symbol when that code span occupies the first table cell; extractor punctuation and internal spaces are preserved
3. `get_spec_symbols` only extracts from `### Exported ...` subsections (allowlist) and top-level tables; skips non-export subsections (e.g., `### API Endpoints`, `### Route Handlers`, `### Configuration`) and `####` method/constructor/properties sub-tables
4. Symbols are deduplicated while preserving order
5. `get_missing_sections` uses regex matching for `## SectionName` headings — case-sensitive
6. Frontmatter parsing handles both scalar fields (module, version, status) and list fields (files, db_tables, depends_on)
7. Empty list syntax `[]` is handled correctly, producing an empty Vec
8. `get_near_miss_sections` only reports sections that are already in `get_missing_sections` — it does not flag sections that are present but close to another required name
9. `parse_checked_issue_references` parses the complete frontmatter as real YAML, permits comments
   and valid trailing commas, and accepts only top-level `implements`/`tracks` sequences of positive
   unsigned issue numbers.
10. Blank, null, scalar, mapping, mixed, zero, negative, and overflowing known issue-reference
    values are rejected with stable content-free errors.
11. Duplicate `implements`/`tracks` keys, duplicate keys elsewhere in the YAML mapping tree, and
    malformed YAML anywhere in frontmatter reject the complete issue-reference parse.
12. Nested extension mappings/sequences and block-scalar text that contain issue-like key names are
    valid YAML but do not contribute issue references.
13. `parse_frontmatter` normalizes CRLF to LF itself and returns an LF-only `body`, so no caller has to. Normalization is guarded on the presence of a carriage return, so an LF document allocates nothing; a lone carriage return unaccompanied by a line feed is content and is preserved. This is a property of the parser, not an obligation on callers: no repository-wide normalize-then-parse convention exists, and the call sites that did not normalize are exactly how a Windows checkout came to fail on every spec.
14. `strip_frontmatter` is the single canonical stripper for the whole repository. Frontmatter ends at its CLOSING delimiter LINE — never at the next `---` anywhere in the document, because `---` is a legal Markdown horizontal rule and a body truncated at one is indistinguishable from a body nobody wrote. It is correct on six axes together: LF, CRLF, a leading BOM, unterminated frontmatter (the whole document is kept rather than a guess), a closing delimiter at end of file with no trailing newline, and a horizontal rule in the body. It borrows rather than allocating, so a CRLF body is returned with its carriage returns intact; a caller needing LF normalizes its own input or reads through `parse_frontmatter`.
15. No module outside `parser` defines its own frontmatter stripper. A second implementation diverges silently in both directions — unstripped frontmatter renders as noise, over-stripped frontmatter deletes body content, and neither raises an error.
16. A frontmatter delimiter LINE is exactly three dashes followed by nothing but whitespace. That one rule governs BOTH ends of the block, in either line encoding, and the two ends need not agree with each other; all three readers in this module — `strip_frontmatter`, `parse_frontmatter`, and `parse_checked_issue_references` — apply it. Trailing whitespace is tolerated because an author cannot see it and refusing it failed silently in both directions: a padded OPENER left the YAML in the body, where a caller counting prose read it as content and `change`'s approval gate passed an artifact with nothing written in it, while a padded CLOSER sent the scan past the real end of the block so that frontmatter ran on to the first horizontal rule in the body and the prose above it was deleted.
17. Leading whitespace, four or more dashes, and any non-dash character make a line NOT a delimiter, and that half of invariant 16 is the load-bearing half. `----` is a legal Markdown thematic break; a document opening with one is a document, and treating it as frontmatter would run the scan forward to the next rule and return a body cut at it — the failure this reader exists to prevent, and one that reads exactly like prose nobody wrote. The residual is stated rather than guessed at: a document opened with `----`, `--- x`, or `---change: x` is returned whole by `strip_frontmatter`, so a caller counting prose still sees its YAML as content.
18. `parse_frontmatter` returns an LF-only body even when only the BODY carries CRLF. Normalization is decided by the presence of a carriage return anywhere in the document, not by the frontmatter's own line endings, so a document with LF frontmatter and a CRLF body — which returned CRLF before normalization moved into the parser — now returns LF. Every consumer is read-only analysis that never maps the body back to raw file bytes; a consumer that needs to must normalize or re-read for itself.

### SPEC SECTION Behavioral Examples

### Scenario: Parse valid frontmatter

- **Given** a spec file with `---\nmodule: auth\nversion: 1\nstatus: stable\nfiles:\n  - src/auth.ts\n---\n`
- **When** `parse_frontmatter(content)` is called
- **Then** returns `Some(ParsedSpec)` with module="auth", version="1", files=["src/auth.ts"]

### Scenario: Parse a CRLF spec from a Windows checkout

- **Given** the same spec file with every line ending rewritten to CRLF by `core.autocrlf=true`
- **When** `parse_frontmatter(content)` is called
- **Then** returns the same frontmatter, with `files` free of a trailing carriage return, and a `body` containing none either

### Scenario: No frontmatter delimiters

- **Given** a plain markdown file without `---` delimiters
- **When** `parse_frontmatter(content)` is called
- **Then** returns `None`

### Scenario: Strip frontmatter from a document with a horizontal rule

- **Given** a document whose frontmatter is followed by a body containing one or more `---` horizontal rules
- **When** `strip_frontmatter(text)` is called
- **Then** only the frontmatter block is removed, and every body rule and the prose around it survives

### Scenario: Strip frontmatter closed at end of file

- **Given** a document that is exactly `---\nmodule: a\n---` with no trailing newline
- **When** `strip_frontmatter(text)` is called
- **Then** returns an empty body rather than the unstripped document

### Scenario: A delimiter carrying trailing whitespace

- **Given** a document whose opening or closing `---` carries a trailing space or tab
- **When** `strip_frontmatter`, `parse_frontmatter`, or `parse_checked_issue_references` reads it
- **Then** the block opens and closes on that line in every reader, so no YAML is left in the body and no body prose is consumed as frontmatter

### Scenario: A document opening with a four-dash thematic break

- **Given** a document whose first line is `----`, followed by prose and a later `---` rule
- **When** `strip_frontmatter(text)` is called
- **Then** the whole document is returned, because a thematic break is not a delimiter and cutting the body at the next rule would delete the prose between them

### Scenario: Extract symbols from Public API

- **Given** a spec body with a table row `| \`createAuth\` | config | Auth | Creates auth |`
- **When** `get_spec_symbols(body)` is called
- **Then** includes "createAuth" in the returned vector

### Scenario: Preserve a GitHub Actions YAML path

- **Given** a recognized Public API table row `| \`inputs.working-directory\` | Working directory |`
- **When** `get_spec_symbols(body)` is called
- **Then** includes the complete symbol "inputs.working-directory" without truncating at punctuation

### Scenario: Parse checked issue references

- **Given** real YAML frontmatter containing `implements: [41,] # comment`, a block `tracks` list,
  and nested extension data containing issue-like keys
- **When** `parse_checked_issue_references(content)` is called
- **Then** only the valid top-level positive unsigned issue IDs are returned

### Scenario: Reject ambiguous issue-reference YAML

- **Given** frontmatter with a duplicate key, malformed extension YAML, or a blank/null/wrong-shaped
  top-level `implements` or `tracks`
- **When** `parse_checked_issue_references(content)` is called
- **Then** the complete parse fails with a stable content-free error

### SPEC SECTION Error Cases

| Condition | Behavior |
|-----------|----------|
| No frontmatter delimiters | `parse_frontmatter` returns `None`; `strip_frontmatter` returns the document unchanged |
| CRLF line endings | Accepted by both readers; never a parse failure and never a surviving frontmatter block |
| Delimiter line padded with trailing whitespace | Still a delimiter in every reader, at either end of the block, so no YAML is left in the body and no body prose is consumed as frontmatter |
| Delimiter line that is not exactly three dashes (`----`, `--- x`, an indented `---`) | Not a delimiter in any reader: `strip_frontmatter` returns the document unchanged, `parse_frontmatter` returns `None`, and `parse_checked_issue_references` returns its stable content-free error |
| Unterminated frontmatter | `parse_frontmatter` returns `None`; `strip_frontmatter` returns the whole document rather than guessing where the block ended |
| Unsupported or malformed content on the compatibility path | `parse_frontmatter` preserves its supported-subset behavior; unknown keys are ignored and missing fields remain `None` |
| Missing/malformed real-YAML frontmatter on the checked issue path | `parse_checked_issue_references` returns a stable content-free error |
| Empty frontmatter block on the checked issue path | `parse_checked_issue_references` returns its stable content-free error, as it always has; a block that is a single blank line still parses as no references |
| Duplicate YAML key anywhere in checked frontmatter | Complete issue-reference parsing fails |
| Blank, null, scalar, mapping, mixed, zero, negative, or overflowing known issue value | Complete issue-reference parsing fails |
| No `## Public API` section | `get_spec_symbols` returns empty vector |
| Empty, unterminated, later-column, or prose backtick span | No symbol is extracted |
| Empty body | `get_missing_sections` reports all required sections as missing |
