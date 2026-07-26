---
spec: cmd_import.spec.md
---

## Key Decisions

- `cmd_import` is a router. It inspects its flags first: `--all-issues` and `--from-dir` short-circuit to batch handlers (`cmd_import_all_issues`, `cmd_import_from_dir`) before the single-import path requires `source` + `id`.
- Single import fails hard (`process::exit(1)`) on any problem; batch modes are resilient — each item that errors increments a counter and the loop continues, ending with a `BatchStats` summary.
- Repo resolution order is identical for single and batch GitHub imports: `--repo` flag → `config.github.repo` → `github::detect_repo(root)`.
- Directory imports use a dedicated preservation path rather than mapping local documents to `ImportSource::Confluence`. A complete valid spec keeps its exact bytes and declared module identity; incomplete/plain Markdown keeps its body while missing contract sections are appended.
- Directory source ownership prefers detected source code and falls back to the imported project-relative Markdown path. An external document with no detected project source fails instead of creating a known-invalid `files: []` spec.
- Import-specific frontmatter validation rejects duplicate keys and wrong scalar/list shapes before any output path is created. It deliberately leaves broader parser unification to the checked-frontmatter work tracked separately.
- Companion generation always runs after a successful spec write; `design.md` is gated on `config.companions.design`.

## Files to Read First

- `src/commands/import.rs` — the router plus `cmd_import_single`, `cmd_import_all_issues`, `cmd_import_from_dir`, and the Markdown parsing helpers.
- `src/importer.rs` — `import_github_issue` / `import_jira_issue` / `import_confluence_page`, `render_spec`, `ImportedItem`, `slugify`, `extract_requirements_pub`.
- `src/github.rs` — `detect_repo`, `list_issues`.
- `src/generator.rs` — `generate_companion_files_for_spec`.
- `src/config.rs` — `load_config`, `companions.design`.

## Current Status

Implemented and stable. Directory preservation, strict-check compatibility, malformed-input atomicity, repo guidance, and the no-args error path are covered by integration tests; remote GitHub/Jira/Confluence fetches are not exercised in CI (network/mocking required).

## Notes

- `cmd_import` routes over three modes from its 7-argument signature: a single GitHub/Jira/Confluence item, all open issues (`all_issues`/`label`), or a directory of files (`from_dir`); `source`/`id` are optional accordingly.
- Part of the command layer — orchestrates importer/github/generator modules rather than containing domain logic.
