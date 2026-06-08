---
spec: cmd_import.spec.md
---

## Key Decisions

- `cmd_import` is a router. It inspects its flags first: `--all-issues` and `--from-dir` short-circuit to batch handlers (`cmd_import_all_issues`, `cmd_import_from_dir`) before the single-import path requires `source` + `id`.
- Single import fails hard (`process::exit(1)`) on any problem; batch modes are resilient — each item that errors increments a counter and the loop continues, ending with a `BatchStats` summary.
- Repo resolution order is identical for single and batch GitHub imports: `--repo` flag → `config.github.repo` → `github::detect_repo(root)`.
- Markdown directory items are mapped to `ImportSource::Confluence` as the closest semantic match for a generic "doc", and the module name is derived from the filename via `importer::slugify`.
- Companion generation always runs after a successful spec write; `design.md` is gated on `config.companions.design`.

## Files to Read First

- `src/commands/import.rs` — the router plus `cmd_import_single`, `cmd_import_all_issues`, `cmd_import_from_dir`, and the Markdown parsing helpers.
- `src/importer.rs` — `import_github_issue` / `import_jira_issue` / `import_confluence_page`, `render_spec`, `ImportedItem`, `slugify`, `extract_requirements_pub`.
- `src/github.rs` — `detect_repo`, `list_issues`.
- `src/generator.rs` — `generate_companion_files_for_spec`.
- `src/config.rs` — `load_config`, `companions.design`.

## Current Status

Implemented and stable. Directory-import and the no-args error path are covered by integration tests; remote GitHub/Jira/Confluence fetches are not exercised in CI (network/mocking required).

## Notes

- Spec frontmatter in `cmd_import.spec.md` documents the original 4-argument single-import signature; the live signature is 7 arguments to accommodate the `--all-issues`/`--label`/`--from-dir` batch modes. The spec is the scored reference; this companion reflects the current code.
- Part of the command layer — orchestrates importer/github/generator modules rather than containing domain logic.
