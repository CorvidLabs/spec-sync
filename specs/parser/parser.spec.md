---
module: parser
version: 10
status: stable
files:
  - src/parser.rs
db_tables: []
tracks: [117]
depends_on:
  - specs/types/types.spec.md
  - specs/util/util.spec.md
---

# Parser

## Purpose

Parses spec markdown files — extracts supported frontmatter into structured data, extracts
backtick-quoted symbol names from Public API tables, and checks for required markdown sections.
The compatibility `parse_frontmatter` path retains its line-oriented supported-subset parser.
Security-sensitive GitHub issue discovery uses a separate maintained `serde-saphyr` real-YAML
parser that validates top-level issue-reference fields and rejects ambiguous or malformed YAML.

## Public API

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
| `get_duplicate_spec_symbols` | Find duplicate symbols in a spec body |
| `is_boilerplate_line` | Detect placeholder documentation lines |

## Invariants

1. `parse_frontmatter` returns `None` if the content does not start with `---\n...\n---\n`
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

## Behavioral Examples

### Scenario: Parse valid frontmatter

- **Given** a spec file with `---\nmodule: auth\nversion: 1\nstatus: stable\nfiles:\n  - src/auth.ts\n---\n`
- **When** `parse_frontmatter(content)` is called
- **Then** returns `Some(ParsedSpec)` with module="auth", version="1", files=["src/auth.ts"]

### Scenario: No frontmatter delimiters

- **Given** a plain markdown file without `---` delimiters
- **When** `parse_frontmatter(content)` is called
- **Then** returns `None`

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

## Error Cases

| Condition | Behavior |
|-----------|----------|
| No frontmatter delimiters | `parse_frontmatter` returns `None` |
| Unsupported or malformed content on the compatibility path | `parse_frontmatter` preserves its supported-subset behavior; unknown keys are ignored and missing fields remain `None` |
| Missing/malformed real-YAML frontmatter on the checked issue path | `parse_checked_issue_references` returns a stable content-free error |
| Duplicate YAML key anywhere in checked frontmatter | Complete issue-reference parsing fails |
| Blank, null, scalar, mapping, mixed, zero, negative, or overflowing known issue value | Complete issue-reference parsing fails |
| No `## Public API` section | `get_spec_symbols` returns empty vector |
| Empty, unterminated, later-column, or prose backtick span | No symbol is extracted |
| Empty body | `get_missing_sections` reports all required sections as missing |

## Dependencies

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
| commands/check | `get_near_miss_sections` (via `fix_near_miss_required_headers`) |
| cmd_issues | `parse_checked_issue_references` for fail-closed CLI issue inspection |
| mcp | `parse_frontmatter` for listing specs; `parse_checked_issue_references` for issue verification |

**Frontmatter Synchronization**

Implementation SHALL add these canonical dependency specs to `depends_on`: `specs/util/util.spec.md`. This YAML frontmatter update is an explicit implementation edit because semantic section deltas do not apply frontmatter.

## Change Log

| Date | Change |
|------|--------|
| 2026-03-25 | Initial spec |
| 2026-06-11 | Add `get_all_api_table_symbols` so `check --fix` treats symbols documented under any Public API table (e.g. a bare `### Functions` heading) as already documented |
| 2026-07-11 | CHG-0010-canonicalize-every-specsync-5-0-contract-and-requirement: Canonicalize every SpecSync 5.0 contract and requirement |
| 2026-07-11 | CHG-0013-preserve-punctuated-public-api-symbols-across-all-export-extractors: Preserve complete punctuated symbols in Public API table rows |
| 2026-07-11 | CHG-0013-preserve-punctuated-public-api-symbols-across-all-export-extractors: Preserve punctuated Public API symbols across all export extractors |
| 2026-07-22 | CHG-0063: Add maintained real-YAML checked issue-reference parsing with duplicate/malformed YAML rejection, strict top-level shapes, CRLF compatibility, and extension-safe semantics |
| 2026-07-27 | CHG-0063-close-independent-mcp-security-review-gaps-for-issue-414: Close independent MCP security review gaps for issue 414 |
| 2026-07-31 | CHG-0070-land-pre-6-0-product-fixes-for-hooks-init-coverage-naming-and-exit-codes: Land pre-6.0 product fixes for hooks init coverage naming and exit codes |
| 2026-08-01 | CHG-0071-land-pre-6-0-product-fixes-for-hooks-init-coverage-naming-and-exit-codes-scoped: Land pre-6.0 product fixes for hooks init coverage naming and exit codes (scoped paths) |
| 2026-08-13 | CHG-0108-stop-reporting-success-for-checks-that-did-not-happen-gate-drafts-that-document: Stop reporting success for checks that did not happen: gate drafts that document a contract over present source, drop cold-cache drift noise, and stop taking quoted frontmatter paths literally |
