## ADDED

### REQUIREMENT REQ-cmd-import-001

The import command SHALL create non-overwriting draft specs from supported single and batch sources with deterministic companion generation.

Acceptance Criteria
- `cmd_import` routes to one of three modes: single import (`source` + `id`), batch issues (`--all-issues`), or batch directory (`--from-dir <dir>`).
- Single import supports sources `github`/`gh`, `jira`, and `confluence`/`wiki`; an unknown source exits 1 with the supported list.
- GitHub repo is resolved from `--repo`, then `github.repo` in config, then `github::detect_repo(root)`; if none resolve, exits 1.
- Each created spec lives at `<specsDir>/<module>/<module>.spec.md` and is never overwritten — an existing spec causes exit 1 (single) or a skip (batch).
- After writing a spec, companions are generated via `generator::generate_companion_files_for_spec` with `companions.design` from config controlling whether `design.md` is created.
- Batch modes print a `[n/total]` progress line per item and a final summary of imported/skipped/error counts; directory mode scans `.md` files one level deep, sorted.
- Markdown directory items derive: title from the first `# ` heading (else filename), purpose from the first non-empty paragraph after the title, requirements via `importer::extract_requirements_pub`, and module name via `importer::slugify(filename)`.
