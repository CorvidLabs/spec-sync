---
change: decide-canonical-materialization-from-the-artefacts-instead-of-the-canonical-applied-flag-so-a-delta-corrected-after
artifact: tasks
---

# Tasks

- [x] Establish what every read and write of `canonical_applied` does before changing what the
      flag means, including the three digest-normalization sites that clear it before hashing and
      the historical-state reconstruction helper that is not a live path.
- [x] Extract `markdown_block_range` from `apply_markdown_block` so applying an item and asking
      whether an item is already applied read the same block out of the same scan.
- [x] Add `delta_item_is_applied`: the question the applier cannot answer, because it applies.
- [x] Add `changelog_records_change`: the durable evidence that this change's version bump and
      Change Log row were both written for this module.
- [x] Replace `prepare_delta_application` with `prepare_pending_delta_application`, which reports
      which modules are outstanding and rewrites only those, giving each only the halves it is
      missing.
- [x] Make the `materialize_change_deltas` short-circuit conditional on there being nothing
      outstanding, and keep the CI refusal below it, now naming the modules.
- [x] Give acceptance the same treatment, since it is the second place deltas reach the canonical
      spec.
- [x] Name `specsync change check` as the second step in both refusals of
      `ensure_approved_delta_bodies_unchanged`.
- [x] Write the discriminators and the control, and fail them against a binary built from a
      separate clean checkout of unfixed `main`.
- [x] Measure the blast radius: parse every archived `state.json` and `approvals.json`, and check
      the Change Log row marker against all archived (change, module) pairs.
- [x] `cargo fmt --check`, `cargo clippy -- -D warnings`, full suite.
