---
spec: cmd_report.spec.md
---

## Tasks

## Post-5.0 Test Debt

- [ ] Add integration tests for `specsync report` (text + JSON, stale detection, incomplete detection, status filtering)

## Done

- [x] Implement `cmd_report` with per-module coverage, staleness, and completeness analysis
- [x] Text table output + stale/incomplete detail sections
- [x] JSON output with per-module detail arrays
- [x] Wire global `--exclude-status` / `--only-status` filtering through `filter_by_status`
- [x] Adopt the resolve-spec-commit-once + `git_commits_since` per-source-file staleness path (N+1 fix)
- [x] Fail closed with structured JSON when checked manifest discovery is inconclusive — Evidence: `malformed_gradle_is_inconclusive_for_coverage_gating_commands`.

## Gaps

- Healthy/stale report rendering still lacks focused integration coverage; malformed-discovery behavior is covered end to end. Staleness logic is only exercised indirectly via `git_utils` tests.

## Review Status

Per-module role sign-offs were not collected. Release approval is governed by digest-bound change approvals and required CI; this note is informational and is not a release gate.
