---
spec: parser.spec.md
---

## Automated Coverage

| Area | Command | Assertions To Watch |
|------|---------|---------------------|
| `src/parser.rs` | cargo test parser:: | `test_parse_frontmatter_basic`, `test_strip_yaml_comment`, `test_parse_frontmatter_inline_comments`, `test_parse_frontmatter_tabs_and_whitespace`, `test_parse_frontmatter_trailing_spaces`, `test_parse_frontmatter_missing` |
| `src/parser.rs` | cargo test parser::checked_issue_references | LF/CRLF delimiters and real-YAML comments/trailing commas are accepted; duplicate/global malformed YAML and blank/null/wrong shapes fail; nested extensions and block scalars do not contribute references; errors do not echo content |
| `src/parser.rs` | cargo test parser::tests::all_frontmatter_readers_agree_on_what_a_delimiter_is | All three readers give the SAME verdict on a matrix of delimiter shapes — this is what fails if the rule in `is_frontmatter_delimiter` and the rule in `FRONTMATTER_RE` ever drift apart |
| `src/parser.rs` | cargo test parser::tests::test_strip_frontmatter_accepts_a_delimiter_padded_with_trailing_whitespace | A padded OPENER opens the block, so its YAML never counts as prose; a padded CLOSER ends it, so body prose above the first horizontal rule is not deleted |
| `src/parser.rs` | cargo test parser::tests::test_strip_frontmatter_refuses_a_delimiter_that_is_not_three_dashes | `----`, `--- x`, `---change: x` and an indented `---` are NOT delimiters — loosening this deletes the body of any document that opens with a thematic break |
| `src/parser.rs` | cargo test parser::tests::test_parse_frontmatter_body_is_lf_even_when_only_the_body_is_crlf | `parsed.body` is LF-only when the frontmatter is LF and the body is CRLF |
| `src/parser.rs` | cargo test parser::tests::test_get_spec_symbols_preserves_complete_punctuated_symbols | Dots, hyphens, selectors, operators, apostrophes, spaces, Unicode, and ordinary identifiers are preserved exactly |
| `src/parser.rs` | cargo test parser::tests::test_api_table_symbol_parser_rejects_empty_or_malformed_rows | Empty, whitespace-only, unterminated, later-column, trailing-text, and prose spans remain excluded |
| `tests/integration.rs` | cargo test --test integration check_github_actions_yaml_with_dotted_exports_passes_strict | Active GitHub Actions workflow contract reports `10/10 exports documented` with zero warnings under strict forced validation |
| `tests/integration.rs` | cargo test --test integration check_undocumented_export_warns | End-to-end fixture: `check_undocumented_export_warns` |
| `tests/integration.rs` | cargo test --test integration check_phantom_export_errors | End-to-end fixture: `check_phantom_export_errors` |
| `tests/integration.rs` | cargo test --test integration invalid_frontmatter_reports_error | End-to-end fixture: `invalid_frontmatter_reports_error` |
| `tests/integration.rs` | cargo test --test integration missing_required_sections_reports_error | End-to-end fixture: `missing_required_sections_reports_error` |
| `tests/integration.rs` | cargo test --test integration missing_frontmatter_fields_reports_error | End-to-end fixture: `missing_frontmatter_fields_reports_error` |
| `tests/integration.rs` | cargo test --test integration fix_adds_undocumented_exports_to_spec | End-to-end fixture: `fix_adds_undocumented_exports_to_spec` |

## Behavioral Verification

| Flow | Fixture / Setup | Action | Expected Result |
|------|-----------------|--------|-----------------|
| Parse valid frontmatter | a spec file with `---\nmodule: auth\nversion: 1\nstatus: stable\nfiles:\n  - src/auth.ts\n---\n` | `parse_frontmatter(content)` is called | returns `Some(ParsedSpec)` with module="auth", version="1", files=["src/auth.ts"] |
| No frontmatter delimiters | a plain markdown file without `---` delimiters | `parse_frontmatter(content)` is called | returns `None` |
| Extract symbols from Public API | a spec body with a table row `\| \`createAuth\` \| config \| Auth \| Creates auth \|` | `get_spec_symbols(body)` is called | includes "createAuth" in the returned vector |
| Extract a punctuated YAML path | a recognized table row with `inputs.working-directory` in the first code span | `get_spec_symbols(body)` is called | returns the complete dotted and hyphenated symbol |

## Regression Matrix

| Case | Required Behavior | Test Obligation |
|------|-------------------|-----------------|
| No frontmatter delimiters | `parse_frontmatter` returns `None` | Keep or add a focused assertion before changing this behavior |
| Unsupported content on compatibility `parse_frontmatter` path | Unknown keys are ignored and missing fields remain `None` within the established subset | Keep focused compatibility assertions separate from checked issue parsing |
| Checked issue YAML contains duplicate keys or malformed unknown extensions | Complete checked parse fails with a stable content-free error | Keep focused duplicate/global-malformed regressions |
| Checked `implements`/`tracks` is blank, null, scalar, mapping, mixed, zero, negative, or overflowing | Complete checked parse fails; invalid entries cannot be filtered away | Keep focused known-field shape/number regressions |
| Checked YAML uses comments or a valid trailing comma | Valid top-level positive unsigned lists are accepted | Keep valid-YAML compatibility regressions |
| Checked YAML uses CRLF frontmatter delimiters | Parsed identically to LF without weakening complete-document validation | Keep the focused CRLF regression plus CLI/MCP caller regressions |
| A delimiter line carries trailing whitespace, at either end of the block | Still a delimiter in all three readers; no YAML is left in the body and no body prose is consumed as frontmatter | Keep the padded-delimiter regressions in `parser` and the artifact-gate regressions in `change` |
| A delimiter line is not exactly three dashes (`----`, `--- x`, indented `---`) | NOT a delimiter in any reader; the document is returned whole | Keep the refusal test — loosening it truncates any document that opens with a thematic break at its next rule |
| The three readers disagree about a delimiter shape | Impossible; the matrix test fails | Never add a fourth frontmatter reader, and never edit `FRONTMATTER_RE`'s delimiter classes without `is_frontmatter_delimiter` |
| Only the BODY of a document is CRLF | `parse_frontmatter` still returns an LF-only body | Keep the mixed-line-ending body assertion; a consumer that maps the body back to raw file bytes must normalize or re-read for itself |
| Nested extension or block-scalar text contains issue-like keys | Nested/text lookalikes are ignored; only top-level fields are authoritative | Keep extension/block-scalar regression |
| No `## Public API` section | `get_spec_symbols` returns empty vector | Keep or add a focused assertion before changing this behavior |
| Malformed or misplaced code span | Empty, unterminated, later-column, and prose spans are ignored | Keep the focused malformed-row regression |
| Empty body | `get_missing_sections` reports all required sections as missing | Keep or add a focused assertion before changing this behavior |

## Reviewer Checklist

- Run the narrow source command above before the full suite when changing `src/parser.rs`.
- Reproduce one Behavioral Verification row with a temporary project fixture before changing user-visible output.
- If an error message changes, update the matching Regression Matrix row and test assertion in the same commit.
- Run the release checks for this module: `fledge run fmt`, `fledge run lint`, `fledge run test`, `fledge spec check --strict`, `./target/release/specsync score --all`.
