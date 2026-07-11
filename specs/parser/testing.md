---
spec: parser.spec.md
---

## Automated Coverage

| Area | Command | Assertions To Watch |
|------|---------|---------------------|
| `src/parser.rs` | cargo test parser:: | `test_parse_frontmatter_basic`, `test_strip_yaml_comment`, `test_parse_frontmatter_inline_comments`, `test_parse_frontmatter_tabs_and_whitespace`, `test_parse_frontmatter_trailing_spaces`, `test_parse_frontmatter_missing` |
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
| Malformed YAML in frontmatter | Unknown keys silently ignored, missing fields remain as `None` | Keep or add a focused assertion before changing this behavior |
| No `## Public API` section | `get_spec_symbols` returns empty vector | Keep or add a focused assertion before changing this behavior |
| Malformed or misplaced code span | Empty, unterminated, later-column, and prose spans are ignored | Keep the focused malformed-row regression |
| Empty body | `get_missing_sections` reports all required sections as missing | Keep or add a focused assertion before changing this behavior |

## Reviewer Checklist

- Run the narrow source command above before the full suite when changing `src/parser.rs`.
- Reproduce one Behavioral Verification row with a temporary project fixture before changing user-visible output.
- If an error message changes, update the matching Regression Matrix row and test assertion in the same commit.
- Run the release checks for this module: `fledge run fmt`, `fledge run lint`, `fledge run test`, `fledge spec check --strict`, `./target/release/specsync score --all`.
