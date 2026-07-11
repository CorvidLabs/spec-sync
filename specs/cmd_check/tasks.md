---
spec: cmd_check.spec.md
---

## Tasks

- [ ] Add a CLI integration test for `check --stale` against a temp git repo (current stale-subcommand tests don't cover `check --stale`)
- [ ] Add an integration test asserting the hash cache skips unchanged specs on a second run

## Done

- [x] Initial spec creation with all required sections
- [x] Requirements and acceptance criteria documented (now reflecting cache, `--fix`, `--backup`, `--dry-run`, `--stale`, formats)
- [x] `--fix` paths covered by integration tests: add/dedupe exports, create Public API section, near-miss header fixes, JSON output, dry-run, backup
- [x] Validation outcomes covered: valid project, missing source file, undocumented export warn, phantom export error
- [x] Git staleness migrated to `git_commits_since` (N+1 fix over the old `git_commits_between`)
- [x] Remove embedded regeneration so `--fix` is deterministic and local

## Gaps

- `src/commands/check.rs` has no inline `#[cfg(test)]` module; coverage is via `tests/integration.rs`
- No integration test directly drives `check --stale` (only the standalone `stale` subcommand is tested)

## Review Sign-offs

- **Product**: pending
- **QA**: pending
- **Design**: n/a
- **Dev**: pending
