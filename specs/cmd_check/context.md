---
spec: cmd_check.spec.md
---

## Key Decisions

- `cmd_check` is the default command and the most feature-dense one: caching, requirements-drift detection, multi-pass `--fix`, git staleness, multiple output formats, and GitHub issue creation all live here.
- Hash-cache skipping is bypassed whenever correctness demands a full pass: `--force`/`--no-cache`, `--strict`, or explicit spec filters.
- `--fix` is intentionally ordered: header renames first (so section/export detection sees canonical headings), then undocumented-export insertion, then AI regeneration for requirements-drifted specs.
- `--backup` aborts the whole `--fix` run on any copy failure rather than risk a partial, unrecoverable rewrite.
- Git staleness uses `git_commits_since(root, spec_commit, source_file)` — one `rev-list` per source file — replacing the old `git_commits_between` pairwise walk (the N+1 fix).
- AI for `--fix` regeneration is resolved through the reworked `ai` module (corvid-ai backed): an Ollama-default provider ladder with `claude` aliased to `anthropic`.
- The cache is only persisted when there are zero errors, so a failing run never "blesses" a broken spec as up-to-date.

## Files to Read First

- `src/commands/check.rs` — the full command: caching, drift prompts, `auto_fix_specs`, `fix_near_miss_headers`, `auto_regen_stale_specs`, git-staleness loop, format dispatch
- `src/commands/mod.rs` — shared `load_and_discover`, `run_validation`, `compute_exit_code`, `exit_with_status`, `create_drift_issues`
- `src/git_utils.rs` — `is_git_repo`, `git_last_commit_hash`, `git_commits_since` (staleness)
- `src/ai.rs` — `resolve_ai_provider`, `regenerate_spec_with_ai` (the `--fix` regen path)
- `src/hash_cache.rs` — `HashCache`, `classify_all_changes`, `update_cache`

## Current Status

Fully implemented and stable. Exercised end-to-end by many `tests/integration.rs` fixtures (validation outcomes, `--fix` variants, `--backup`, `--dry-run`, JSON). No inline `#[cfg(test)]` module in `check.rs` itself.

## Notes

- Orchestrates library modules; domain logic (validation, scoring, AI, git) lives elsewhere.
- `--dry-run` without `--fix` prints a warning and otherwise does nothing.
