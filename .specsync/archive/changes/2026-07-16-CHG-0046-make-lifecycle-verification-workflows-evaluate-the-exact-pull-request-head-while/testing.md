---
change: CHG-0046-make-lifecycle-verification-workflows-evaluate-the-exact-pull-request-head-while
artifact: testing
---

# Testing

- Parse both workflow files as YAML and assert the lifecycle checkout steps have
  `fetch-depth: 0` and the exact-head/fallback expression.
- Assert representative ordinary CI checkout steps have no explicit `ref`, preserving
  synthetic-merge validation.
- Run strict SpecSync validation at 100% file and LOC coverage.
- Verify the affected lifecycle changes at one common exact commit.
- Run the candidate-path Trust gate and record progressive provenance honestly.
- Preserve the existing hosted PR #387 failure as evidence of the pre-fix boundary; a
  hosted rerun remains required after integration.
