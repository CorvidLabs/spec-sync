---
spec: cmd_import.spec.md
---

## Key Decisions

- `cmd_import` is a router. It inspects its flags first: `--all-issues` and `--from-dir` short-circuit to batch handlers (`cmd_import_all_issues`, `cmd_import_from_dir`) before the single-import path requires `source` + `id`.
- Single import fails hard (`process::exit(1)`) on any problem; batch modes are resilient — each item that errors increments a counter and the loop continues, ending with a `BatchStats` summary.
- Repo resolution order is identical for single and batch GitHub imports: `--repo` flag → `config.github.repo` → `github::detect_repo(root)`.
- GitHub reads require explicit `GITHUB_TOKEN` and stay in process. Single issue imports delegate to `importer::import_github_issue`; batch imports use `github::list_issues`, whose strict pagination rejects malformed links, duplicates, and cap truncation.
- GitHub pagination is bounded to 100 pages of 100 issues and each REST operation is bounded to 10 seconds; authenticated `gh` state is intentionally not a compatibility fallback.
- Markdown directory items are mapped to `ImportSource::Confluence` as the closest semantic match for a generic "doc", and the module name is derived from the filename via `importer::slugify`.
- Companion generation always runs after a successful spec write; `design.md` is gated on `config.companions.design`.

## Files to Read First

- `src/commands/import.rs` — the router plus `cmd_import_single`, `cmd_import_all_issues`, `cmd_import_from_dir`, and the Markdown parsing helpers.
- `src/importer.rs` — `import_github_issue` / `import_jira_issue` / `import_confluence_page`, `render_spec`, `ImportedItem`, `slugify`, `extract_requirements_pub`.
- `src/github.rs` — `detect_repo`, strict typed `list_issues` pagination and operation bounds.
- `src/generator.rs` — `generate_companion_files_for_spec`.
- `src/config.rs` — `load_config`, `companions.design`.

## Current Status

Implemented and stable. Directory-import and the no-args error path are covered by integration tests. GitHub response parsing, pagination, duplicate/cap rejection, 404 revalidation, and provider-process exclusion are covered by isolated unit tests; live remote GitHub/Jira/Confluence fetches are not exercised in CI.

## Notes

- `cmd_import` routes over three modes from its 7-argument signature: a single GitHub/Jira/Confluence item, all open issues (`all_issues`/`label`), or a directory of files (`from_dir`); `source`/`id` are optional accordingly.
- Part of the command layer — orchestrates importer/github/generator modules rather than containing domain logic.
