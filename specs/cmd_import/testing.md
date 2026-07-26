---
spec: cmd_import.spec.md
---

## Automated Coverage

| Area | Command | Assertions To Watch |
|------|---------|---------------------|
| `src/commands/import.rs` | cargo test commands::import::tests | Focused conversion tests cover byte preservation, confined source ownership, empty-field repair, malformed frontmatter, suffix handling, and portable dates. |
| `src/commands/import.rs` | cargo test --test integration import_ | Directory-import + no-args/repo-guidance paths are covered end-to-end; GitHub/Jira/Confluence fetches are not (network/mocking required). |
| `tests/integration.rs` | cargo test --test integration import_without_args_or_flags_shows_error | Missing source/id with no batch flag fails with "SOURCE is required". |
| `tests/integration.rs` | cargo test --test integration import_from_dir_imports_markdown_files | `--from-dir docs` prints "Batch Import" and reports 1 imported for one `.md` file. |
| `tests/integration.rs` | cargo test --test integration import_from_dir_skips_existing_specs | A pre-existing `my-feature` spec causes the same-named doc to be skipped. |
| `tests/integration.rs` | cargo test --test integration import_from_dir_nonexistent_directory_errors | `--from-dir nonexistent-dir` fails with "Directory not found". |
| `tests/integration/languages.rs` | cargo test --test integration import_from_dir_preserves_complete_spec_bytes_and_declared_module | Complete CRLF spec output is byte-identical and uses its frontmatter module for the destination. |
| `tests/integration/languages.rs` | cargo test --test integration import_from_dir_rejects_malformed_frontmatter_without_output | Duplicate and wrong-shape fields fail the batch and create no module directory. |
| `tests/integration/languages.rs` | cargo test --test integration import_github_repo_error_points_to_current_config | Missing-repo guidance names `[github] repo` in `.specsync/config.toml`. |

## Behavioral Verification

| Flow | Fixture / Setup | Action | Expected Result |
|------|-----------------|--------|-----------------|
| Single GitHub import | repo resolvable, issue exists | `specsync import github 42` | Fetches issue #42, writes `specs/<module>/<module>.spec.md` + companions, prints "Imported:" and a `specsync check` tip. |
| Unknown source | any | `specsync import slack 7` | Exits 1: "Unknown source 'slack'. Supported: github, jira, confluence". |
| Batch issues | repo resolvable | `specsync import --all-issues --label bug` | Prints "Batch Import: GitHub Issues", per-issue `[n/total]` lines, and an imported/skipped/error summary. |
| Batch directory | `docs/*.md` present | `specsync import --from-dir docs` | One spec per Markdown file (one level deep, sorted), existing specs skipped; generated `files` owns detected code or the project-relative document and passes strict validation. |
| Existing complete spec | valid frontmatter and all required sections | `specsync import --from-dir docs` | Output bytes are identical and destination uses the declared module. |

## Regression Matrix

| Case | Required Behavior | Test Obligation |
|------|-------------------|-----------------|
| Missing source/id, no batch flag | Exits 1 with "SOURCE is required" guidance | Covered by `import_without_args_or_flags_shows_error`. |
| Unknown source type | Exits 1 with the supported-source list | Keep; add a focused assertion before changing the message. |
| Non-numeric GitHub issue id | Exits 1 with "Invalid issue number" | Add a fixture before changing id parsing. |
| Spec already exists (single) | Exits 1 | Add a focused assertion before changing this behavior. |
| Spec already exists (batch) | Skipped, counted, loop continues | Covered by `import_from_dir_skips_existing_specs`. |
| `--from-dir` missing directory | Exits 1 with "Directory not found" | Covered by `import_from_dir_nonexistent_directory_errors`. |
| Complete spec uses CRLF or filename differs from declared module | Preserve exact bytes and use declared module destination | Covered by `import_from_dir_preserves_complete_spec_bytes_and_declared_module`. |
| Empty, unterminated, duplicate-key, or wrong-shape frontmatter | Exits nonzero after batch and creates no output for the bad item | Covered by unit and integration malformed-input tests. |
| No matching source code | Use the project-relative source Markdown file; never emit `files: []` | Covered by `import_from_dir_imports_markdown_files`, including a follow-up check asserting frontmatter and source-file validity. |
| Input document resolves outside the project and no project source matches | Fail the item rather than recording an escaping/absolute ownership path | Covered by `from_dir_fails_loudly_on_unparseable_input`. |
| GitHub repo cannot be detected | Guidance names `.specsync/config.toml`, not legacy `specsync.json` | Covered by `import_github_repo_error_points_to_current_config`. |
| Fetch fails (single) | Exits 1 with the upstream error | Add a fixture/mocked path before changing error handling. |

## Reviewer Checklist

- Run `cargo run -- import --help` and confirm `--repo`, `--all-issues`, `--label`, and `--from-dir` are present.
- Run `cargo test --test integration import_` before the full suite when changing `src/commands/import.rs`.
- Reproduce one Behavioral Verification row with a temp project before changing user-visible output (progress lines, summary, tips).
- If an error message changes, update the matching Regression Matrix row and assertion in the same commit.
- Run the release checks for this module: `fledge run fmt`, `fledge run lint`, `fledge run test`, `fledge spec check --strict`, `fledge run build`.
