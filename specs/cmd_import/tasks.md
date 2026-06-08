---
spec: cmd_import.spec.md
---

## Tasks

- [ ] Add integration coverage for single GitHub import (currently only the no-args error and `--from-dir` flows are tested; GitHub/Jira/Confluence paths require network or mocking).
- [ ] Add a fixture asserting `companions.design` toggles `design.md` generation on import.

## Done

- [x] Single import for `github`/`gh`, `jira`, `confluence`/`wiki` with repo resolution (flag → config → `detect_repo`).
- [x] Batch import of open GitHub issues (`--all-issues`, optional `--label`) with per-item progress and summary.
- [x] Batch import of a Markdown directory (`--from-dir`), non-recursive, sorted, with title/purpose/requirements extraction.
- [x] Companion generation wired to `companions.design` config flag.
- [x] Success guidance updated to tell users to validate/complete imported details (not fill template markers).
- [x] Integration coverage: `import_without_args_or_flags_shows_error`, `import_from_dir_imports_markdown_files`, `import_from_dir_skips_existing_specs`, `import_from_dir_nonexistent_directory_errors`.

## Review Sign-offs

- **Product**: pending
- **QA**: pending
- **Design**: n/a
- **Dev**: pending
