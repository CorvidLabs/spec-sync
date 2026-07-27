---
spec: git_utils.spec.md
---

## Key Decisions

- **Git CLI wrapper**: shells out to `git` via `std::process::Command` rather than linking libgit2, keeping the dependency tree small.
- **Safe defaults**: git failures return `None` / `0` / `false` so callers decide whether to warn, skip, or fail.
- **Repository-root execution**: every command runs with `current_dir(root)`.
- **N+1 fix (2026-06-07)**: the old `git_commits_between` (which re-resolved the spec commit on every call) was removed. The current shape is: caller resolves the spec's commit once with `git_last_commit_hash`, then calls `git_commits_since(root, spec_commit, source_file)` per source file. `git_commits_since` runs `git rev-list --count {spec_commit}..HEAD -- {source_file}`.

## Public Surface

- `git_last_commit_hash(root, file) -> Option<String>`
- `git_commits_since(root, spec_commit, source_file) -> usize`
- `is_git_repo(root) -> bool`
- `struct StaleInfo { spec_path, module_name, max_commits_behind, source_details }`

## Files to Read First

- `src/git_utils.rs` — the helpers, `StaleInfo`, and the inline `#[cfg(test)]` module.
- `src/commands/stale.rs` — the primary caller demonstrating the resolve-once / count-per-file pattern.

## Current Status

Stable. Used by staleness detection, reports, check, and scoring freshness. It first checks whether
the current source bytes match the spec commit, avoiding add/revert false positives, then counts
commits only for content that still differs. Covered by inline unit tests using `tempfile` + real
`git` invocations.

## Notes

- Do not make these helpers panic on git errors; graceful degradation is part of the public behavior.
