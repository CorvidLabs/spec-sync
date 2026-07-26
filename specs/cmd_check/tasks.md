---
spec: cmd_check.spec.md
---

## Tasks

- [x] Add an integration test asserting the hash cache skips unchanged specs on a second run — Evidence: `check_skips_unchanged_specs`.
- [x] Replay complete cached errors, warnings, and notices without mutating diagnostic strings
- [x] Report full checked, freshly validated, and cached spec counts in JSON
- [x] Force re-validation for malformed, incompatible, stale, tampered, or incomplete snapshots
- [x] Add warm/cold parity, strict bypass, ignore/inventory invalidation, and rehash regressions for issue #429

## Post-5.0 Test Debt

- [ ] Add a CLI integration test for `check --stale` against a temp git repo (current stale-subcommand tests don't cover `check --stale`)

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

## Review Status

Per-module role sign-offs were not collected. Release approval is governed by digest-bound change approvals and required CI; this note is informational and is not a release gate.
