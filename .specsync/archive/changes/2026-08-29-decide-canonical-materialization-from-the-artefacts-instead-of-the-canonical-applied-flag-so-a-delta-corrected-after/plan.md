---
change: decide-canonical-materialization-from-the-artefacts-instead-of-the-canonical-applied-flag-so-a-delta-corrected-after
artifact: plan
---

# Plan

1. **Survey the flag before changing what it means.** Every read and write of `canonical_applied`
   in `src/change.rs`, classified: live gates (`materialize_change_deltas`, `accept_change_with_gate`,
   `finalize_change`, `add_acceptance_owner_corrections`, `correct_interview_metadata`,
   `ensure_reopened_definition_unchanged`, `ensure_no_delta_conflicts`), state transitions
   (`approve_definition_with_projection`, `reopen_change`), and the three digest-normalization
   sites plus one historical-state reconstruction helper that clear the flag before hashing and are
   NOT live paths.
2. **Extract `markdown_block_range`** from `apply_markdown_block` and re-express the applier in
   terms of it, so the applier and the new predicate share one scan.
3. **Add the two predicates** — `delta_item_is_applied` and `changelog_records_change`.
4. **Replace `prepare_delta_application` with `prepare_pending_delta_application`**, returning
   `(files, pending)` and rewriting only the modules with something outstanding, giving each only
   the halves it is missing. Scope convergence to `record.canonical_applied`.
5. **Make the two short-circuits conditional** — in `materialize_change_deltas` and in
   `accept_change_with_gate` — and keep the CI refusal below the new question so CI still refuses
   to materialize, now naming the modules that are behind.
6. **Fix the diagnostic** in both refusals of `ensure_approved_delta_bodies_unchanged`.
7. **Discriminate.** Write the tests, build a binary from a separate clean checkout of unfixed
   `main`, add only the tests to it, and record verbatim failures. Include the CONTROL and label
   it honestly.
8. **Measure the blast radius** rather than argue it: parse every archived `state.json` and
   `approvals.json`, and check the Change Log row marker across archived (change, module) pairs.
9. **Gates**: `cargo fmt --check`, `cargo clippy -- -D warnings`, full `cargo test`,
   `change check`, `change audit --strict`.

## Sequencing note

The whole source edit was completed before `change new`, then stashed, the change created with its
final `--spec`/`--path` set, and the edit restored — because delivery scope freezes at `change new`
(#542) and a scope discovered later cannot be added to an approved change.
