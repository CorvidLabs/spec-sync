## MODIFIED

### SPEC SECTION Invariants

1. Promote/demote use `SpecStatus::next()` / `SpecStatus::prev()` for linear transitions
2. `set` validates transitions via `SpecStatus::can_transition_to()` unless `--force` is used
3. Status updates are written in-place by regex-replacing the `status:` line within frontmatter delimiters only (never in body)
4. Single spec is resolved via `filter_specs` — exits 1 if ambiguous or no match
5. JSON output uses `OutputFormat::Json` for machine-readable results
6. Transition guards check min_score, require_sections, and staleness
6a. A `no_stale` guard that git cannot evaluate FAILS rather than passes: with no committed history
    the guard's question has no answer, and an unanswerable guard must not be reported as satisfied
    (#572). A spec that git simply has no record of yet is a different case — there is nothing for
    it to be behind, so the guard passes, matching `stale`'s treatment since #558.
7. Lifecycle history is appended to frontmatter `lifecycle_log` when `track_history` is enabled
8. Git reads in this module run through `git_utils::output_after_child_exits`, so a read returns when the Git child exits even while a descendant still holds its output pipe (REQ-git-utils-008).
