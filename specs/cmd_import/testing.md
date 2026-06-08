---
spec: cmd_import.spec.md
---

## Automated Coverage

| Area | Command | Assertions To Watch |
|------|---------|---------------------|
| `src/commands/import.rs` | cargo test --test integration import_ | No inline `#[cfg(test)]` module. Directory-import + no-args paths are covered end-to-end; GitHub/Jira/Confluence fetches are not (network/mocking required). |
| `tests/integration.rs` | cargo test --test integration import_without_args_or_flags_shows_error | Missing source/id with no batch flag fails with "SOURCE is required". |
| `tests/integration.rs` | cargo test --test integration import_from_dir_imports_markdown_files | `--from-dir docs` prints "Batch Import" and reports 1 imported for one `.md` file. |
| `tests/integration.rs` | cargo test --test integration import_from_dir_skips_existing_specs | A pre-existing `my-feature` spec causes the same-named doc to be skipped. |
| `tests/integration.rs` | cargo test --test integration import_from_dir_nonexistent_directory_errors | `--from-dir nonexistent-dir` fails with "Directory not found". |

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

## Reviewer Checklist

- Run `cargo run -- import --help` and confirm `--repo`, `--all-issues`, `--label`, and `--from-dir` are present.
- Run `cargo test --test integration import_` before the full suite when changing `src/commands/import.rs`.
- Reproduce one Behavioral Verification row with a temp project before changing user-visible output (progress lines, summary, tips).
- If an error message changes, update the matching Regression Matrix row and assertion in the same commit.
- Run the release checks for this module: `fledge run fmt`, `fledge run lint`, `fledge run test`, `fledge spec check --strict`, `fledge run build`.
