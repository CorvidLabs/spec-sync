---
change: required-ci-gate-fails-when-the-lifecycle-gate-fails
artifact: plan
---

# Plan

Four files change, plus the `github` spec and its companions.

1. **`.github/workflows/ci.yml`.** Add `preflight` and `lifecycle-gate` to
   `implementation-gate.needs`. Replace its step with the selection-aware `GATES` table and check.
   Run the new test in `validate-action`. Correct the `ci-gate` comment, which said lifecycle
   coherence is proven in spec-check alone.
2. **`.github/scripts/test-required-ci-gate.py`.** The structural guard, the job-graph
   simulation, the pre-#796 reproduction, guard mutation tests and a `--truth-table` mode for the
   pull request.
3. **`fledge.toml`.** Add the `ci-gate-test` task to the `verify`, `ci` and `repo` lanes.
4. **`docs/HLD.md`.** State what `SpecSync implementation ready` requires, next to the existing
   note that branch protection requires `Required CI gate`.
5. **`github` spec.** Add the gate invariant and REQ-github-021 through the delta. Record the
   test in `testing.md`, the decision in `context.md` and the task in `tasks.md`.
6. Show that the test fails on the pre-fix `ci.yml`, then run `fledge lanes run verify`,
   `fledge lanes run pre-push` and `fledge trust verify`.
7. Open the pull request before approval. The unapproved draft fails `Lifecycle gate`, and with
   this change `Required CI gate` should go red on it, which is the live proof.
