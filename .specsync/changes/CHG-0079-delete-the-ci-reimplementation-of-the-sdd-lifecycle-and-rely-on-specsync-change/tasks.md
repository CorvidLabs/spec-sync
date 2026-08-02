---
change: CHG-0079-delete-the-ci-reimplementation-of-the-sdd-lifecycle-and-rely-on-specsync-change
artifact: tasks
---

# Tasks

- [x] Enumerate deleted assertions and classify each as covered or deliberately dropped
- [x] Confirm `lifecycle-validation-limits.json` is read by `src/change.rs:329` and retain it
- [x] Confirm `src/change.rs:2493` names a path pattern rather than reading the workflow file
- [x] Delete the six lifecycle scripts and three lifecycle workflows
- [x] Remove the `archive-integrity` and `scoped-review-reuse` jobs from `ci.yml`
- [x] Remove the metadata-child classification and ancestor-reuse steps from `trust.yml`
- [x] Add `cargo run -- change audit --strict` to `spec-check`
- [x] Reduce `ci-gate` to one aggregate context without the forced archive-tip failure
- [x] Update `specs/github/context.md` and `specs/github/testing.md`
- [x] Re-run retained validators: classify-ci-paths, workflow-runtime-pins, validate-release-candidate
- [x] Run the full Rust suite
