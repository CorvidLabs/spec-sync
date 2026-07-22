---
spec: cmd_import.spec.md
---

## User Stories

- As a developer, I want to turn a GitHub/Jira/Confluence item into a spec draft with companions so that I can start specs from existing tracking artifacts instead of from scratch.
- As a maintainer onboarding a repo, I want to batch-import all open GitHub issues (optionally filtered by label) so that I can seed specs for the whole backlog in one pass.
- As a writer with a folder of design docs, I want to batch-import a directory of Markdown files so that each doc becomes a spec draft.
- As a developer, I want the import to tell me to validate and complete the imported details (not fill template markers) so that the next step is clear.

## Acceptance Criteria

- `cmd_import` routes to one of three modes: single import (`source` + `id`), batch issues (`--all-issues`), or batch directory (`--from-dir <dir>`).
- Single import supports sources `github`/`gh`, `jira`, and `confluence`/`wiki`; an unknown source exits 1 with the supported list.
- GitHub repo is resolved from `--repo`, then `github.repo` in config, then `github::detect_repo(root)`; if none resolve, exits 1.
- Single and batch GitHub imports require explicit `GITHUB_TOKEN`; authenticated `gh` state is not a read fallback and no provider subprocess is launched.
- `--all-issues` follows strict GitHub pagination for at most 100 pages of 100 issues and fails on malformed links, duplicate issue IDs, or a continuing next page at the cap instead of importing a partial list.
- Every GitHub REST operation is bounded to 10 seconds.
- Each created spec lives at `<specsDir>/<module>/<module>.spec.md` and is never overwritten — an existing spec causes exit 1 (single) or a skip (batch).
- After writing a spec, companions are generated via `generator::generate_companion_files_for_spec` with `companions.design` from config controlling whether `design.md` is created.
- Batch modes print a `[n/total]` progress line per item and a final summary of imported/skipped/error counts; directory mode scans `.md` files one level deep, sorted.
- Markdown directory items derive: title from the first `# ` heading (else filename), purpose from the first non-empty paragraph after the title, requirements via `importer::extract_requirements_pub`, and module name via `importer::slugify(filename)`.

## Constraints

- GitHub issue `id` must parse as `u64`; a non-numeric id exits 1 (single import).
- Must not panic on unreadable files or fetch failures: single import exits 1; batch modes count the failure and continue.
- Directory scan is non-recursive (one level deep).

## Out of Scope

- Two-way sync back to the external system; this is a one-shot import.
- Filling in the full spec body — imported specs are drafts to be completed and validated with `specsync check`.
- Recursive directory traversal and non-Markdown source files.

### REQ-cmd-import-001

The import command SHALL create non-overwriting draft specs from supported single and batch sources with deterministic companion generation.

Acceptance Criteria
- `cmd_import` routes to one of three modes: single import (`source` + `id`), batch issues (`--all-issues`), or batch directory (`--from-dir <dir>`).
- Single import supports sources `github`/`gh`, `jira`, and `confluence`/`wiki`; an unknown source exits 1 with the supported list.
- GitHub repo is resolved from `--repo`, then `github.repo` in config, then `github::detect_repo(root)`; if none resolve, exits 1.
- Single and batch GitHub imports require explicit `GITHUB_TOKEN`; authenticated `gh` state is not a read fallback and no provider subprocess is launched.
- `--all-issues` follows strict GitHub pagination for at most 100 pages of 100 issues and fails on malformed links, duplicate issue IDs, or a continuing next page at the cap instead of importing a partial list.
- Every GitHub REST operation is bounded to 10 seconds.
- Each created spec lives at `<specsDir>/<module>/<module>.spec.md` and is never overwritten — an existing spec causes exit 1 (single) or a skip (batch).
- After writing a spec, companions are generated via `generator::generate_companion_files_for_spec` with `companions.design` from config controlling whether `design.md` is created.
- Batch modes print a `[n/total]` progress line per item and a final summary of imported/skipped/error counts; directory mode scans `.md` files one level deep, sorted.
- Markdown directory items derive: title from the first `# ` heading (else filename), purpose from the first non-empty paragraph after the title, requirements via `importer::extract_requirements_pub`, and module name via `importer::slugify(filename)`.

