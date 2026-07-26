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
- Each created spec lives at `<specsDir>/<module>/<module>.spec.md` and is never overwritten — an existing spec causes exit 1 (single) or a skip (batch).
- After writing a spec, companions are generated via `generator::generate_companion_files_for_spec` with `companions.design` from config controlling whether `design.md` is created.
- Batch modes print a `[n/total]` progress line per item and a final summary of imported/skipped/error counts; directory mode scans `.md` files one level deep, sorted.
- Markdown directory items with complete valid spec frontmatter preserve their bytes and declared module identity. Plain or incomplete Markdown preserves its body and is supplemented only with missing required frontmatter and sections.
- Directory imports never emit an empty `files` list: matching source code is preferred and the canonically confined project-relative input document is the fallback. If neither is safe and available, the item fails without output.
- Empty input, unterminated frontmatter, duplicate keys, and known-field scalar/list shape mismatches are reported as errors; the batch exits nonzero after processing the remaining files.
- A missing GitHub repository tells users to pass `--repo` or configure `repo` under `[github]` in `.specsync/config.toml`.

## Constraints

- GitHub issue `id` must parse as `u64`; a non-numeric id exits 1 (single import).
- Must not panic on unreadable files or fetch failures: single import exits 1; batch modes count the failure and continue.
- Directory scan is non-recursive (one level deep).

## Out of Scope

- Two-way sync back to the external system; this is a one-shot import.
- Rewriting complete valid specs or replacing imported prose with generated boilerplate.
- Recursive directory traversal and non-Markdown source files.

### REQ-cmd-import-001

The import command SHALL create non-overwriting spec artifacts from supported single and batch sources with deterministic companion generation, while preserving the status and bytes of complete valid directory-imported specs.

Acceptance Criteria
- `cmd_import` routes to one of three modes: single import (`source` + `id`), batch issues (`--all-issues`), or batch directory (`--from-dir <dir>`).
- Single import supports sources `github`/`gh`, `jira`, and `confluence`/`wiki`; an unknown source exits 1 with the supported list.
- GitHub repo is resolved from `--repo`, then `github.repo` in config, then `github::detect_repo(root)`; if none resolve, exits 1.
- Each created spec lives at `<specsDir>/<module>/<module>.spec.md` and is never overwritten — an existing spec causes exit 1 (single) or a skip (batch).
- After writing a spec, companions are generated via `generator::generate_companion_files_for_spec` with `companions.design` from config controlling whether `design.md` is created.
- Batch modes print a `[n/total]` progress line per item and a final summary of imported/skipped/error counts; directory mode scans `.md` files one level deep, sorted.
- Markdown directory items with complete valid spec frontmatter preserve their bytes and declared module identity. Plain or incomplete Markdown preserves its body and is supplemented only with missing required frontmatter and sections.
- Directory imports never emit an empty `files` list: matching source code is preferred and the canonically confined project-relative input document is the fallback. If neither is safe and available, the item fails without output.
- Empty input, unterminated frontmatter, duplicate keys, and known-field scalar/list shape mismatches are reported as errors; the batch exits nonzero after processing the remaining files.
- A missing GitHub repository tells users to pass `--repo` or configure `repo` under `[github]` in `.specsync/config.toml`.
