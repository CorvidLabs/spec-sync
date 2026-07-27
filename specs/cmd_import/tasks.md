---
spec: cmd_import.spec.md
---

## Tasks

## Post-5.0 Test Debt

- [ ] Add recorded integration coverage for a successful GitHub import; missing-token single and
  batch command paths are covered without network access.
- [ ] Add a fixture asserting `companions.design` toggles `design.md` generation on import.

## Done

- [x] Single import for `github`/`gh`, `jira`, `confluence`/`wiki` with repo resolution (flag → config → `detect_repo`).
- [x] Batch import of open GitHub issues (`--all-issues`, optional `--label`) with per-item progress and summary.
- [x] Batch import of a Markdown directory (`--from-dir`), non-recursive, sorted, with title/purpose/requirements extraction.
- [x] Companion generation wired to `companions.design` config flag.
- [x] Success guidance updated to tell users to validate/complete imported details (not fill template markers).
- [x] Integration coverage: `import_without_args_or_flags_shows_error`, `import_from_dir_imports_markdown_files`, `import_from_dir_skips_existing_specs`, `import_from_dir_nonexistent_directory_errors`.
- [x] Require explicit-token in-process GitHub reads for single and batch import; remove authenticated `gh` read fallback.
- [x] Follow strict bounded GitHub pagination and fail closed on malformed links, duplicate issue IDs, or cap truncation.
- [x] Prove single and batch GitHub imports fail without a token and create no spec output.
- [x] Validate every single/batch output module name before filesystem writes
- [x] Continue batch imports after item failures while returning exit 1 for any error, with partial
  success and unsafe-path regression coverage

## Review Status

Per-module role sign-offs were not collected. Release approval is governed by digest-bound change approvals and required CI; this note is informational and is not a release gate.
