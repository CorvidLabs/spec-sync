---
spec: ignore.spec.md
---

## Automated Coverage

| Area | Command | Assertions To Watch |
|------|---------|---------------------|
| `src/ignore.rs` | cargo test ignore:: | `test_classify_requirements_companion`, `test_classify_stub_section`, `test_classify_undocumented_export`, `test_classify_schema_type_before_column`, `test_from_str_aliases`, `test_parse_inline` |
| `tests/integration.rs` | cargo test --test integration init_ignores_node_modules_and_hidden_dirs | End-to-end fixture: `init_ignores_node_modules_and_hidden_dirs` |

## Behavioral Verification

| Flow | Fixture / Setup | Action | Expected Result |
|------|-----------------|--------|-----------------|
| Global suppression | `.specsyncignore` contains `requirements-companion` | a spec triggers "Missing companion requirements.md" warning | `is_suppressed()` returns true for any spec path |
| Per-spec path suppression | `.specsyncignore` contains `stub-section:specs/legacy/` | spec `specs/legacy/api.spec.md` has a Purpose section with no substantive content | warning is suppressed |
| Inline directive | spec body contains `<!-- specsync-ignore: undocumented-export, changelog -->` | `parse_inline()` is called | returns set containing `UndocumentedExport` and `ChangelogEntries` |

## Regression Matrix

| Case | Required Behavior | Test Obligation |
|------|-------------------|-----------------|
| `.specsyncignore` does not exist | Returns empty `IgnoreRules` (not an error) | Keep or add a focused assertion before changing this behavior |
| Unrecognized category string | Silently skipped during load; `from_str()` returns `None` | Keep or add a focused assertion before changing this behavior |
| Malformed inline comment (missing `-->`) | Directive is ignored | Keep or add a focused assertion before changing this behavior |
| Warning text doesn't match any pattern | `classify()` returns `None`, warning is never suppressed | Keep or add a focused assertion before changing this behavior |

## Reviewer Checklist

- Run the narrow source command above before the full suite when changing `src/ignore.rs`.
- Reproduce one Behavioral Verification row with a temporary project fixture before changing user-visible output.
- If an error message changes, update the matching Regression Matrix row and test assertion in the same commit.
- Run the release checks for this module: `fledge run fmt`, `fledge run lint`, `fledge run test`, `fledge spec check --strict`.
