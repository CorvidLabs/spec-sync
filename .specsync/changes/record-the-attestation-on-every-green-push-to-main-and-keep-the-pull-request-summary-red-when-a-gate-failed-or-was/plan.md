---
change: record-the-attestation-on-every-green-push-to-main-and-keep-the-pull-request-summary-red-when-a-gate-failed-or-was
artifact: plan
---

# Plan

## Approach

Failing test first, through the existing workflow model. Then make the two workflow edits: `attest`'s `if:` gets `always()`, and the summary rows carry selections, with two new rows. Then wire the test in and record REQ-github-029 and REQ-github-030 in the `github` spec.

## Out of scope

- The corvid-pet Action itself (`CorvidLabs/corvid-pet@v1.0.0`).
- `attest`'s signing step.
- `implementation-gate` and `ci-gate`.
- Any other job.
- No tag or release.

## Steps

1. Commit CI-9, CI-10, the failing `test-ci-attest-and-summary.py` and this definition; record approval.
2. Edit `attest`'s `if:` and the summary step in `ci.yml`.
3. Add fledge task `ci-attest-and-summary-test` to the `ci`, `verify` and `repo` lanes, and a `validate-action` step.
4. `specs/github`: materialize REQ-github-029 and REQ-github-030, add a testing row and a `github.3md` lesson plane.
5. Run `specsync change check --commit`, `specsync change audit --strict`, pre-push and verify. Then review, ship, archive and push; CI must be green. The first push to main after merge should show `Record attestation` running.

## Risks

- **First live attestation.** The first real run of `attest` since it was added comes after merge. It is best-effort, so a download or push hiccup only warns.
- **Merge overlap with G-608 (#1007).** It also edits `ci.yml`, `fledge.toml` and `hi/ci.md`. The conflicts are list and row insertions, resolved at merge.

## Constraints consulted

Required CI gate and `implementation-gate` keep their contract (#796). The summary uses the same selection rule, checked against the same model. Archive-first CI. Ruby Psych workflow loading. Never weaken a gate or test.
