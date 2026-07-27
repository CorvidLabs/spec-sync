## MODIFIED

### REQUIREMENT REQ-parser-001

The parser SHALL deterministically parse the supported frontmatter and Public API Markdown subset,
SHALL identify required and stub sections, and SHALL provide fail-closed real-YAML parsing for
security-sensitive GitHub issue references.

Acceptance Criteria

- `parse_frontmatter` retains its established supported-subset compatibility behavior.
- `parse_checked_issue_references` parses the complete frontmatter document with maintained
  `serde-saphyr` real-YAML semantics and duplicate-key rejection.
- LF and CRLF frontmatter delimiters are accepted equivalently.
- YAML comments and valid trailing commas are accepted.
- Duplicate `implements`/`tracks` keys, duplicate keys elsewhere in the YAML mapping tree, and
  malformed YAML anywhere in frontmatter reject the complete checked parse.
- Top-level `implements` and `tracks` accept only sequences of positive unsigned issue numbers;
  blank, null, scalar, mapping, mixed, zero, negative, and overflowing values are rejected.
- Nested extension mappings/sequences and block-scalar text containing issue-like keys do not
  contribute issue references.
- Checked parse errors are stable and content-free.

### SPEC SECTION Public API

#### Exported Structs

| Type | Description |
|------|-------------|
| `ParsedSpec` | Parsed spec file containing `frontmatter: Frontmatter` and `body: String` |

#### Exported Functions

| Function | Parameters | Returns | Description |
|----------|-----------|---------|-------------|
| `parse_frontmatter` | `content: &str` | `Option<ParsedSpec>` | Parse supported-subset frontmatter delimited by `---` from a spec file |
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
