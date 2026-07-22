---
spec: cmd_import.spec.md
---

## Automated Coverage

| Area | Command | Assertions To Watch |
|------|---------|---------------------|
| `src/commands/import.rs` | cargo test --test integration import_ | Directory-import and no-args paths plus single/batch GitHub missing-token failures are covered end-to-end; live fetch success remains network/recording dependent. |
| `src/github.rs` | cargo test github::tests | Typed issue parsing; complete strict pagination; malformed Link, duplicate-ID, page-cap, 404 revalidation, and no-read-subprocess failures. |
| `src/importer.rs` | cargo test importer::tests | GitHub issue detail conversion delegates to the shared typed REST contract; rendering and non-GitHub import parsing remain stable. |
| `tests/integration.rs` | cargo test --test integration import_without_args_or_flags_shows_error | Missing source/id with no batch flag fails with "SOURCE is required". |
| `tests/integration.rs` | cargo test --test integration import_from_dir_imports_markdown_files | `--from-dir docs` prints "Batch Import" and reports 1 imported for one `.md` file. |
| `tests/integration.rs` | cargo test --test integration import_from_dir_skips_existing_specs | A pre-existing `my-feature` spec causes the same-named doc to be skipped. |
| `tests/integration.rs` | cargo test --test integration import_from_dir_nonexistent_directory_errors | `--from-dir nonexistent-dir` fails with "Directory not found". |
| `tests/integration.rs` | cargo test --test integration single_github_import_fails_closed_without_a_rest_token_or_output | Single GitHub import exits non-zero with explicit-token guidance and creates no spec. |
| `tests/integration.rs` | cargo test --test integration batch_github_import_fails_closed_without_a_rest_token_or_output | Batch GitHub import exits non-zero with explicit-token guidance and creates no specs. |

## Behavioral Verification

| Flow | Fixture / Setup | Action | Expected Result |
|------|-----------------|--------|-----------------|
| Single GitHub import | repo resolvable, issue exists | `specsync import github 42` | Fetches issue #42, writes `specs/<module>/<module>.spec.md` + companions, prints "Imported:" and a `specsync check` tip. |
| Unknown source | any | `specsync import slack 7` | Exits 1: "Unknown source 'slack'. Supported: github, jira, confluence". |
| Batch issues | repo resolvable | `specsync import --all-issues --label bug` | Prints "Batch Import: GitHub Issues", per-issue `[n/total]` lines, and an imported/skipped/error summary. |
| Batch directory | `docs/*.md` present | `specsync import --from-dir docs` | One spec per Markdown file (one level deep, sorted), existing specs skipped. |

## Regression Matrix

| Case | Required Behavior | Test Obligation |
|------|-------------------|-----------------|
| Missing source/id, no batch flag | Exits 1 with "SOURCE is required" guidance | Covered by `import_without_args_or_flags_shows_error`. |
| Unknown source type | Exits 1 with the supported-source list | Keep; add a focused assertion before changing the message. |
| Non-numeric GitHub issue id | Exits 1 with "Invalid issue number" | Add a fixture before changing id parsing. |
| Spec already exists (single) | Exits 1 | Add a focused assertion before changing this behavior. |
| Spec already exists (batch) | Skipped, counted, loop continues | Covered by `import_from_dir_skips_existing_specs`. |
| `--from-dir` missing directory | Exits 1 with "Directory not found" | Covered by `import_from_dir_nonexistent_directory_errors`. |
| Fetch fails (single) | Exits 1 with the upstream error | Add a fixture/mocked path before changing error handling. |
| `GITHUB_TOKEN` missing | Single and batch GitHub imports fail with explicit-token guidance, never consult `gh`, and create no output | Covered by `single_github_import_fails_closed_without_a_rest_token_or_output`, `batch_github_import_fails_closed_without_a_rest_token_or_output`, and shared provider regressions. |
| Paginated list contains malformed Link data or duplicate IDs | Batch import fails instead of importing a partial/ambiguous issue set | Covered by `link_header_parsing_detects_next_and_rejects_malformed_values` and `issue_list_pagination_fails_instead_of_truncating_or_deduplicating`. |
| Page 100 still advertises a next page | Batch import fails instead of silently truncating | Covered by `issue_list_pagination_fails_instead_of_truncating_or_deduplicating`. |

## Reviewer Checklist

- Run `cargo run -- import --help` and confirm `--repo`, `--all-issues`, `--label`, and `--from-dir` are present.
- Run `cargo test --test integration import_` before the full suite when changing `src/commands/import.rs`.
- Reproduce one Behavioral Verification row with a temp project before changing user-visible output (progress lines, summary, tips).
- Run `cargo test github::tests` and `cargo test importer::tests` when changing GitHub import behavior.
- If an error message changes, update the matching Regression Matrix row and assertion in the same commit.
- Run the release checks for this module: `fledge run fmt`, `fledge run lint`, `fledge run test`, `fledge spec check --strict`, `fledge run build`.
