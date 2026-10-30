---
change: authenticate-finalized-policy-migrations-during-active-only-audit-without-weakening-policy-change-coverage
artifact: tasks
---

# Tasks

- [x] Reproduce the Algonest48 failure and distinguish it from Algonest49's resolved Git hydration issue.
- [x] Prepare and obtain actual human definition approval for the scope, constraints and semantic delta.
- [x] Implement internal audit correction and nine positive/security regression cases.
- [x] Prove the original implementation fails the new positive discriminator.
- [x] Synchronize owning contract testing guidance and observed lessons.

## Verification gates

All native product-lane steps pass, including all Rust tests, optimized build, strict contracts and workflow validators; scoped verification passes. Native Trust, commit-bound provenance and hosted checks are still required before delivery. Exact results and environment corrections are recorded in testing.md.

## Delivery follow-up

Publish the verified upstream PR and exact Algonest reproduction evidence. Hosted checks and genuine provenance remain required. Prepare Algonest48 consumption separately using an immutable verified candidate identity, with all gates preserved. Human implementation review, merge, release and deployment are separate.
