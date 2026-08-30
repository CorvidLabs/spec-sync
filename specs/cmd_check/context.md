---
spec: cmd_check.spec.md
---

## Key Decisions

- `cmd_check` is the default command and the most feature-dense one: caching, requirements-drift detection, multi-pass `--fix`, git staleness, multiple output formats, and GitHub issue creation all live here.
- Hash-cache skipping is bypassed whenever correctness demands a full pass: `--force`/`--no-cache`, `--strict`, or explicit spec filters.
- The cache skips re-validation, not the previous result. Findings are stored as per-spec snapshots and replayed when inputs have not changed. A hash-only cache (no snapshot) is treated as a miss.
- `--fix` is intentionally deterministic: header renames first, then undocumented-export insertion. Requirements drift remains guidance rather than triggering inference.
- `--backup` aborts the whole `--fix` run on any copy failure rather than risk a partial, unrecoverable rewrite.
- Git staleness uses `git_commits_since(root, spec_commit, source_file)` — one `rev-list` per source file — replacing the old `git_commits_between` pairwise walk (the N+1 fix).
- The cache is only persisted when there are zero errors, so a failing run never "blesses" a broken spec as up-to-date.
- `specsync check` does not consult SDD. Path coverage of dirty files by an active change is `change audit`, not this command. `--require-coverage` remains the reverse-coverage gate.
- Every coverage calculation uses `compute_coverage_checked`; malformed Gradle settings fail closed, and JSON mode reports `valid: false` with `inconclusive: true`.

## Files to Read First

- `src/commands/check.rs` — the full command: caching, drift prompts, `auto_fix_specs`, `fix_near_miss_headers`, `fix_near_miss_required_headers`, git-staleness loop, format dispatch. There is no regeneration helper here: `auto_regen_stale_specs` was deleted with embedded inference in 5.0 (#335) and drift is guidance, never a rewrite
- `src/commands/mod.rs` — shared `load_and_discover`, `run_validation`, `compute_exit_code`, `exit_with_status`, `create_drift_issues`
- `src/git_utils.rs` — `missing_history`, `spec_baseline`, `git_commits_since` (staleness)
- `src/hash_cache.rs` — `HashCache`, `classify_all_changes`, `update_cache`

## Current Status

Fully implemented and stable. Exercised end-to-end by many `tests/integration.rs` fixtures (validation outcomes, `--fix` variants, `--backup`, `--dry-run`, JSON). No inline `#[cfg(test)]` module in `check.rs` itself.

## Notes

- Orchestrates deterministic validation, scoring, lifecycle, and git modules.
- `--dry-run` without `--fix` prints a warning and otherwise does nothing.
