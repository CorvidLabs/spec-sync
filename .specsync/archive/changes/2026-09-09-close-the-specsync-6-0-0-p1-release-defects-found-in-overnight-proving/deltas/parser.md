## ADDED

### REQUIREMENT REQ-parser-004

Public API symbol extraction SHALL ignore fenced code. Backtick names inside a fenced example are quoted text, not documented exports.

Acceptance Criteria
- `get_spec_symbols` and `get_all_api_table_symbols` run on fence-blanked text.
- `blank_fenced_code` preserves the original byte length so offsets remain valid for `--fix`.
- A fenced `### Exported Functions` heading does not create an export subsection.

## MODIFIED

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
| `get_spec_symbols` | `body: &str` | `Vec<String>` | Extract backtick-quoted symbol names from the `## Public API` section tables, ignoring fenced examples |
| `get_missing_sections` | `body: &str, required_sections: &[String]` | `Vec<String>` | Check which required `##` sections are missing from the spec body |
| `is_export_header` | `header: &str` | `bool` | Return whether a `###` header denotes an exported-symbols subsection |
| `section_has_content` | `body: &str, section: &str` | `bool` | Return whether the `## Section` block contains substantive content |
| `find_stub_sections` | `body: &str, required_sections: &[String]` | `Vec<String>` | Return required sections that are present but lack substantive content |
| `find_section_offset` | `body: &str, section: &str` | `Option<usize>` | Return the byte offset of an exact `## Section` heading |
| `body_has_section` | `body: &str, section: &str` | `bool` | Return whether the body contains an exact `## Section` heading |
| `get_near_miss_sections` | `body: &str, required_sections: &[String]` | `Vec<(String, String)>` | Return missing canonical sections paired with near-miss headings |
| `get_all_api_table_symbols` | `body: &str` | `Vec<String>` | Extract the first backtick-quoted symbol from every Public API table row, ignoring fenced examples |
| `get_duplicate_spec_symbols` | Find duplicate symbols in a spec body |
| `is_boilerplate_line` | Detect placeholder documentation lines |
| `blank_fenced_code` | `body: &str` | `String` | Space-blank fenced code at the original byte length so Public API readers and `--fix` ignore quoted examples |
