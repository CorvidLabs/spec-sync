# Parser canonical frontmatter reader delta

## MODIFIED

### SPEC SECTION Purpose

Parses spec markdown files — extracts supported frontmatter into structured data, extracts
backtick-quoted symbol names from Public API tables, and checks for required markdown sections.
The compatibility `parse_frontmatter` path retains its line-oriented supported-subset parser.
Security-sensitive GitHub issue discovery uses a separate maintained `serde-saphyr` real-YAML
parser that validates top-level issue-reference fields and rejects ambiguous or malformed YAML.

This module also owns the repository's single frontmatter stripper, `strip_frontmatter`. Both
readers accept CRLF themselves, because there is no caller-side normalization convention to lean
on: the delimiter-recognition rule lives here once rather than in every module that reads a
Markdown file.

### SPEC SECTION Public API

#### Exported Structs

| Type | Description |
|------|-------------|
| `ParsedSpec` | Parsed spec file containing `frontmatter: Frontmatter` and `body: String` |

#### Exported Functions

| Function | Parameters | Returns | Description |
|----------|-----------|---------|-------------|
| `parse_frontmatter` | `content: &str` | `Option<ParsedSpec>` | Parse supported-subset frontmatter delimited by `---` from a spec file, in LF or CRLF, returning an LF-only body |
| `strip_frontmatter` | `text: &str` | `&str` | Return the Markdown body with YAML frontmatter removed, borrowed from the input; the single canonical stripper |
| `parse_checked_issue_references` | `content: &str` | `Result<(Vec<u64>, Vec<u64>), String>` | Parse and strictly validate top-level `implements` and `tracks` issue-reference lists from real YAML frontmatter |
| `get_spec_symbols` | `body: &str` | `Vec<String>` | Extract backtick-quoted symbol names from the `## Public API` section tables |
| `get_missing_sections` | `body: &str, required_sections: &[String]` | `Vec<String>` | Check which required `##` sections are missing from the spec body |
| `is_export_header` | `header: &str` | `bool` | Return whether a `###` header denotes an exported-symbols subsection |
| `section_has_content` | `body: &str, section: &str` | `bool` | Return whether the `## Section` block contains substantive content |
| `find_stub_sections` | `body: &str, required_sections: &[String]` | `Vec<String>` | Return required sections that are present but lack substantive content |
| `find_section_offset` | `body: &str, section: &str` | `Option<usize>` | Return the byte offset of an exact `## Section` heading |
| `body_has_section` | `body: &str, section: &str` | `bool` | Return whether the body contains an exact `## Section` heading |
| `get_near_miss_sections` | `body: &str, required_sections: &[String]` | `Vec<(String, String)>` | Return missing canonical sections paired with near-miss headings |
| `get_all_api_table_symbols` | `body: &str` | `Vec<String>` | Extract the first backtick-quoted symbol from every Public API table row |
| `get_duplicate_spec_symbols` | Find duplicate symbols in a spec body |
| `is_boilerplate_line` | Detect placeholder documentation lines |

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
| Unterminated frontmatter | `parse_frontmatter` returns `None`; `strip_frontmatter` returns the whole document rather than guessing where the block ended |
| Unsupported or malformed content on the compatibility path | `parse_frontmatter` preserves its supported-subset behavior; unknown keys are ignored and missing fields remain `None` |
| Missing/malformed real-YAML frontmatter on the checked issue path | `parse_checked_issue_references` returns a stable content-free error |
| Duplicate YAML key anywhere in checked frontmatter | Complete issue-reference parsing fails |
| Blank, null, scalar, mapping, mixed, zero, negative, or overflowing known issue value | Complete issue-reference parsing fails |
| No `## Public API` section | `get_spec_symbols` returns empty vector |
| Empty, unterminated, later-column, or prose backtick span | No symbol is extracted |
| Empty body | `get_missing_sections` reports all required sections as missing |

### SPEC SECTION Dependencies

**Consumes**

| Module | What is used |
|--------|-------------|
| types | `Frontmatter` struct |
| regex | `Regex`, `LazyLock` for compiled patterns |
| serde | Checked issue-reference deserialization visitors |
| serde-saphyr | Real-YAML parsing with duplicate-key rejection |

**Consumed By**

| Module | What is used |
|--------|-------------|
| validator | `parse_frontmatter`, `get_spec_symbols`, `get_missing_sections`, `get_near_miss_sections` |
| scoring | `parse_frontmatter`, `get_spec_symbols`, `get_missing_sections` |
| view | `parse_frontmatter` for the spec, `strip_frontmatter` for the companion `requirements.md` |
| change | `strip_frontmatter` for lesson counting, archived lesson bundles, and artifact completeness |
| commands/check | `get_near_miss_sections` (via `fix_near_miss_required_headers`) |
| cmd_issues | `parse_checked_issue_references` for fail-closed CLI issue inspection |
| mcp | `parse_frontmatter` for listing specs; `parse_checked_issue_references` for issue verification |

**Frontmatter Synchronization**

Implementation SHALL add these canonical dependency specs to `depends_on`: `specs/util/util.spec.md`. This YAML frontmatter update is an explicit implementation edit because semantic section deltas do not apply frontmatter.
