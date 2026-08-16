---
spec: cmd_check.spec.md
---

## Tasks

- [x] Add an integration test asserting the hash cache skips unchanged specs on a second run — Evidence: `check_skips_unchanged_specs`.
- [x] A warm cache must replay stored findings rather than report a clean skip — Evidence: `warm_cache_text_replays_cached_warnings`, `warm_cache_json_replays_cached_warnings`, `hashes_without_snapshots_revalidate_instead_of_going_silent`.

## Post-5.0 Test Debt

- [ ] Add a CLI integration test for `check --stale` against a temp git repo (current stale-subcommand tests don't cover `check --stale`)

## Done

- [x] Initial spec creation with all required sections
- [x] Requirements and acceptance criteria documented (now reflecting cache, `--fix`, `--backup`, `--dry-run`, `--stale`, formats)
- [x] `--fix` paths covered by integration tests: add/dedupe exports, create Public API section, near-miss header fixes, JSON output, dry-run, backup
- [x] Validation outcomes covered: valid project, missing source file, undocumented export warn, phantom export error
- [x] Git staleness migrated to `git_commits_since` (N+1 fix over the old `git_commits_between`)
- [x] Remove embedded regeneration so `--fix` is deterministic and local
- [x] Fail closed on malformed Gradle coverage discovery and preserve structured JSON failure output — Evidence: `malformed_gradle_is_inconclusive_for_coverage_gating_commands`.

## Gaps

- `src/commands/check.rs` has no inline `#[cfg(test)]` module; coverage is via `tests/integration.rs`
- No integration test directly drives `check --stale` (only the standalone `stale` subcommand is tested)

## Review Status

Per-module role sign-offs were not collected. Release approval is governed by digest-bound change approvals and required CI; this note is informational and is not a release gate.
