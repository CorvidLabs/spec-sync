---
spec: cmd_issues.spec.md
---

## Automated Coverage

| Area | Command | Assertions To Watch |
|------|---------|---------------------|
| `src/commands/issues.rs` | cargo test commands::issues::tests | Pure summary regressions distinguish no references from provider errors and inspection findings; unreadable paths become content-free findings. |
| `tests/integration/commands.rs` | focused `issues_without_references` filters | No-reference projects skip Git/provider access; valid configured identity is preserved and malformed configured identity is rejected. |
| `tests/integration/commands.rs` | cargo test --test integration commands::issues_reference_batch_fails_closed_without_a_rest_token | A referenced issue with configured repo but no token exits non-zero with attributed JSON error output. |
| `tests/integration/commands.rs` | cargo test --test integration commands::issues_retains_unreadable_and_malformed_specs_as_safe_findings | Text, JSON, Markdown, and GitHub retain both finding kinds, suppress no-reference guidance, exit 1, and never disclose fixture bytes. |
| Checked field/path helpers | cargo test commands::issues::tests | Strict real-YAML top-level shapes, duplicate/global malformed YAML, valid comments/trailing commas, ignored nested/block-scalar lookalikes, non-UTF-8 suffix classification, and safe rendering. |
| Capability snapshot regressions | focused command unit/integration filters | Project/spec-directory capabilities, same-handle identity checks, path replacement, spec-shaped non-files, non-UTF-8 names, and escaping/symlinked `specs_dir` fail closed without partial/empty discovery. |
| Snapshot identity and bounds | focused `commands::issues::tests` filters | `discovery_to_read_replacement_hook_rejects_regular_file_replacement`, `discovery_to_read_replacement_hook_rejects_hardlink_replacement`, and `snapshot_discovery_enforces_per_file_cumulative_and_file_count_limits` bind discovered identity through read and enforce all retention ceilings. |
| Shared root identity and total inventory bound | focused `commands::issues::tests` filters | `mapped_sources_use_original_root_capability_after_root_symlink_replacement` keeps spec/source authority on one retained root, while `snapshot_discovery_bounds_huge_non_spec_inventories_before_accumulation` caps all visited entries before accumulation. |
| `--create` snapshot validation | focused command integration filter plus `snapshot_validation_never_reopens_replaced_mapped_source` | Drift validation consumes retained spec/source observations through `validate_spec_content_with_sources`, never reopens replaced paths, and does not resolve supplied-content TypeScript wildcards through ambient paths. |
| Renderer adversarial characters | focused command unit/integration filters | Controls, bidi formatting controls, Zl/Zp separators, pipes, backticks, and newline-like input remain escaped and structurally safe in every output format. |
| Cross-platform relative paths | `cargo test relative_paths_use_slashes_on_every_platform` plus focused `issues_` integration tests | Windows emits forward-slash relative paths and native junction fixtures avoid command-option parsing; Unix preserves literal backslashes in filenames. |
| Checked selected config | focused `issues_*config*` integration tests plus `load_issues_config_checked_with_hooks` unit tests | Malformed/invalid UTF-8/wrong-shaped, linked, replaced, or over-4-MiB config emits a structured finding; exact retained bytes and one project capability govern parsing and specs. |
| Structured early outcomes | `issues_missing_or_empty_specs_use_selected_structured_renderer`, `issues_repository_resolution_failures_use_selected_structured_renderer` | JSON parses and Markdown/GitHub retain structured reports for missing/empty specs and repository errors. |
| `src/github.rs` | cargo test github | Typed classification, global deduplication/cap, strict provider parsing, transport failure, and timeout are covered in the GitHub module. |
| MCP batch cap | cargo test mcp::tests::issue_tool_enforces_one_deduplicated_invocation_cap_across_specs | Multiple individually safe specs exceed the project-wide cap before provider access. |

## Coverage Gaps

- No end-to-end fixture exercises live successful/closed/not-found classification; no-reference
  behavior and explicit-token provider failure are covered without network access.
- Add recorded/mocked GitHub responses to cover the valid/closed/not-found/error classification and the non-zero exit on 404 or error.

## Behavioral Verification

| Flow | Fixture / Setup | Action | Expected Result |
|------|-----------------|--------|-----------------|
| No references | specs have no `implements`/`tracks` | `specsync issues` | Prints "No issue references found in spec frontmatter." and exits 0. |
| All references valid | issues #10, #15, #20 open | `specsync issues` | Prints "3 valid, 0 closed, 0 not found" and exits 0. |
| Closed reference | a referenced issue is closed | `specsync issues` | Warns "(closed — spec may need updating)"; still exits 0 if nothing is not-found/errored. |
| Stale reference (404) | spec references a deleted issue | `specsync issues` | Reports it as not found and exits 1. |
| `--create` with drift | immutable retained spec/source snapshots that fail validation | `specsync issues --create` | Validates through `validate_spec_content_with_sources` and opens drift issues for those failures without reopening discovered paths or ambient wildcard targets. |
| Unreadable and malformed specs | one invalid UTF-8 spec and one missing-frontmatter spec | `specsync issues` in text, JSON, Markdown, and GitHub formats | Reports content-free path-attributed findings, suppresses no-reference guidance, and exits 1. |
| Malformed issue fields | scalar, mapping, mixed, negative, or non-numeric `implements`/`tracks` values | `specsync issues` | Reports malformed frontmatter and exits 1 without provider access. |
| Traversal failure | recursive spec walk encounters an unreadable or disappearing entry | `specsync issues` | Retains an inconclusive discovery finding; never claims no references. |
| Hostile filename | path contains controls, pipes, backticks, and newline-like characters | render every supported format | Paths stay project-relative/content-free; text is control-safe, JSON parses, and Markdown/GitHub emits one valid escaped row and code span. |
| Replaced discovered spec | a path is swapped after discovery | continue issue inspection or `--create` validation | Same-handle validation rejects the race or uses only the retained snapshot; replacement bytes are not trusted. |
| Regular/hardlink replacement | a discovered spec name is swapped to a different regular inode or hardlink before read | continue discovery | Identity mismatch is a safe read finding; replacement bytes are not parsed. |
| Snapshot bounds | one spec exceeds 4 MiB, retained specs exceed 64 MiB, or discovery exceeds 10,000 specs | continue discovery | Bounded findings force exit 1; no unbounded snapshot set is retained. |
| Edge-backtick filename | a finding path starts or ends with a backtick | render Markdown/GitHub | Code-span content is padded and remains one valid escaped table cell. |
| Invalid configured repo with no references | `github.repo` is malformed and all snapshots have no references | `specsync issues` | Exits 1 before no-reference success and performs no Git/provider access. |
| Invalid configured repo with missing/empty specs | `github.repo` is malformed and the specs directory is absent or contains no specs | `specsync issues` | Exits 1 before "No spec files found." and performs no Git/provider access. |

## Regression Matrix

| Case | Required Behavior | Test Obligation |
|------|-------------------|-----------------|
| GitHub repo unresolvable with references | Prints error, exits 1 | Add a focused assertion before changing repo resolution. |
| No references and no configured repository | Guidance, exit 0, no Git/provider access | Keep the unconfigured no-reference regression. |
| No references and configured repository | Validate exact `owner/repository` syntax; valid identity succeeds without Git/provider access and malformed identity exits 1 | Keep valid and invalid configured-repository regressions. |
| Missing/empty specs and configured repository | Validate exact syntax before no-spec success; malformed identity exits 1 | Keep missing-directory and empty-directory configured-repository regressions. |
| Issue returns 404 (not found) | Counted as not-found; triggers exit 1 | Add a mocked fixture before changing exit logic. |
| Verification error (e.g. API/auth failure) | Counted as error; **triggers exit 1** (`total_not_found > 0 || total_errors > 0`) | Missing-token command path is covered by `issues_reference_batch_fails_closed_without_a_rest_token`; retain a recorded provider-error fixture before changing classification. |
| All references produce errors | Summary includes the error count and does not print no-reference guidance | Keep `all_error_batches_report_errors_instead_of_no_reference_guidance`. |
| Duplicate references across specs | One provider lookup per unique ID | Keep the GitHub batch deduplication regression. |
| More than 100 unique references | Batch error before provider access and exit 1 | Keep the MCP cross-spec cap regression. |
| Closed issue only | Warned but does **not** by itself force a non-zero exit | Add a mocked fixture before changing the exit condition. |
| Spec without `implements`/`tracks` | Skipped; Git auto-detection/provider access are skipped when all specs are empty, while configured syntax is still validated | Keep the network-free no-reference fixtures. |
| Spec unreadable or frontmatter malformed/missing | Retained as `read_error` or `malformed_frontmatter`, content omitted, no-reference success suppressed, exit 1 | Keep `issues_retains_unreadable_and_malformed_specs_as_safe_findings` and the focused command unit regressions. |
| `implements`/`tracks` wrong shape or invalid member | Malformed-frontmatter finding; no invalid member is silently filtered | Add checked scalar, mapping, mixed-list, signed, and non-numeric fixtures. |
| Recursive discovery error or spec-shaped non-file entry | Inconclusive finding, not a partial/empty scan | `issues_retains_spec_shaped_discovery_failures` asserts no-reference guidance is absent. |
| Hostile display path | Relative and content-free in every format; no raw controls; Markdown/GitHub code span and table remain valid | Exercise pipes, backticks, controls, and line breaks without asserting host-absolute paths. |
| Bidi or Unicode Zl/Zp display character | Escaped in every renderer; cannot alter visual order or line/table structure | Exercise bidi formatting controls plus U+2028/U+2029. |
| Spec path replacement after discovery | Original same-handle snapshot or inconclusive finding; never replacement content | Keep same-handle identity race coverage. |
| Regular-file or hardlink replacement after discovery | Read finding; replacement bytes are never trusted | Keep both discovery-to-read replacement regressions. |
| Spec snapshot limits | 4 MiB per spec, 64 MiB cumulative, 10,000 specs | Keep `snapshot_discovery_enforces_per_file_cumulative_and_file_count_limits`. |
| Recursive inventory limit | At most 100,000 total entries, including non-spec files | Keep `snapshot_discovery_bounds_huge_non_spec_inventories_before_accumulation`. |
| Ambient project root replaced between spec and source collection | Both remain bound to the original retained project capability | Keep `mapped_sources_use_original_root_capability_after_root_symlink_replacement`. |
| `--create` snapshot validation | Validation and drift decisions use retained immutable spec/source snapshots; no path reopen or ambient wildcard resolution | Keep replacement regressions around both the spec and mapped-source seams. |
| Markdown code span starts/ends with backtick | Pads content inside a longer delimiter | Keep `markdown_code_spans_pad_leading_and_trailing_backticks`. |

## Reviewer Checklist

- Run `cargo run -- issues --help` and confirm `--create` and the format flags are present.
- For changes to verification/classification, run the `github` module's tests — that is where the API logic lives.
- Confirm the exit-code condition (`not_found > 0 || errors > 0 || inspection findings exist`) still matches the documented Regression Matrix before changing it.
- If an output or error message changes, update the matching Regression Matrix row and assertion in the same commit.
- Run the release checks for this module: `fledge run fmt`, `fledge run lint`, `fledge run test`, `fledge spec check --strict`, `fledge run build`.
