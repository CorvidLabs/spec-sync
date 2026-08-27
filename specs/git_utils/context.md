---
spec: git_utils.spec.md
---

## Key Decisions

- **Git CLI wrapper**: shells out to `git` via `std::process::Command` rather than linking libgit2, keeping the dependency tree small.
- **Safe defaults**: git failures return `None` / `0` / `false` so callers decide whether to warn, skip, or fail. "No history" is the one thing that is NOT a safe default — see the choke point below.
- **Repository-root execution**: every command runs with `current_dir(root)`.
- **N+1 fix (2026-06-07)**: the old `git_commits_between` (which re-resolved the spec commit on every call) was removed. The current shape is: caller resolves the spec's baseline once with `spec_baseline`, then calls `git_commits_since(root, spec_commit, source_file)` per source file. `git_commits_since` runs `git rev-list --count {spec_commit}..HEAD -- {source_file}`.
- **Choke point over helper (2026-08-14, #572)**: the commit lookup used to be `git_last_commit_hash -> Option<String>`, and its `None` meant either "history exists, this spec is not in it" (drift genuinely zero) or "there is no history" (drift unknown). Five callers had to tell those apart by remembering to call `is_git_repo` + `has_commits` first; four did not, and reported every spec current on a tree with no `.git`. The alternative considered was a shared `missing_history` guard helper, but a helper is fail-open — the sixth caller forgets it and nothing complains. Instead the raw lookup was made PRIVATE and `spec_baseline` returns a three-way enum, so a caller that ignores the missing case does not compile. `missing_history` remains public only for commands that want to fail fast once before a loop.

## Public Surface

- `spec_baseline(root, spec_file) -> SpecBaseline`
- `missing_history(root) -> Option<MissingHistory>`
- `git_commits_since(root, spec_commit, source_file) -> usize`
- `source_was_deleted(root, since, path) -> bool`
- `is_git_repo(root) -> bool`
- `has_commits(root) -> bool`
- `enum SpecBaseline { Commit(String), Untracked, Missing(MissingHistory) }`
- `enum MissingHistory { NotARepository, NoCommits }` with `reason()` / `sentence()`
- `struct StaleInfo { spec_path, module_name, max_commits_behind, source_details, deleted_files }`

## Files to Read First

- `src/git_utils.rs` — the helpers, `StaleInfo`, and the inline `#[cfg(test)]` module.
- `src/commands/stale.rs` — the primary caller demonstrating the resolve-once / count-per-file pattern.

## Current Status

Stable. Used by staleness detection, reports, check, and scoring freshness. Covered by inline unit tests using `tempfile` + real `git` invocations.

## Notes

- Do not make these helpers panic on git errors; graceful degradation is part of the public behavior.
