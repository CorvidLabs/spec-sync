---
change: record-the-attestation-on-every-green-push-to-main-and-keep-the-pull-request-summary-red-when-a-gate-failed-or-was
artifact: context
---

# Context

Two CI defects from the #796/#797 work, grouped as G-ci-jobs in the 2026-11-06 triage.

- **#798:** `Record attestation` (`attest`) was skipped on every push to main, including green ones.
  - Its `if:` (`needs.ci-gate.result == 'success' && github.event_name == 'push' && github.ref == 'refs/heads/main'`) has no status function, so GitHub ANDs in an implicit `success()`.
  - At job level, `success()` looks at every transitive dependency. `ci-gate` needs `implementation-gate`, which needs `corvid-pet`, and `corvid-pet` runs only on pull requests.
  - So on a push `success()` is false and `attest` is skipped.
  - `test-required-ci-gate.py`'s model already simulates this and says so in a comment.
- **#799:** the scoped review's summary (`corvid-pet`, step `status`) builds its table from `CHECK_*` rows.
  - It has no row for `preflight` or `lifecycle-gate`, and it labels every `skipped` result "Not selected".
  - A failed lifecycle gate skips `test`, `audit`, `coverage` and `spec-check`, and the summary still reads green. After #797 the merge is blocked, but the review comment is misleading.

HI: CI-9 and CI-10 are new, in Leif's words ("Both A", Herdr orc tab, 2026-11-06, relayed by orc t18). REQ-github-029 and REQ-github-030 were claimed with the orc.

Ruled out:
- Dropping `corvid-pet` from `implementation-gate`'s needs. #796 requires every product job that can finish before the gate to be in its needs.
- Moving `attest` off `ci-gate`. Attestation must follow the required gate.
