---
change: a-definition-approval-may-not-withdraw-the-delta-binding-an-earlier-approval-recorded
artifact: plan
---

# Plan

1. Reproduce both halves against the unfixed binary before writing anything: the ledger
   downgrade on workflow v1, and the materialization of unapproved wording from the same shape
   on workflow v2. Record what each produced.
2. `src/change.rs` — `append_portable_definition_approval_v501`: compute
   `delta_body_digests` once, after the projection pair is established so its diagnostics keep
   priority, and record it on both members of the pair.
3. `src/change.rs` — `ensure_approved_delta_bodies_unchanged`: keep the absence path, and
   qualify it. Refuse only when some other `definition`-gate approval in the same ledger records
   a digest, with a message naming `specsync change approve <id>`.
4. `src/change.rs` — state the invariant where the field is declared, so the next reader of
   `approved_delta_digests` finds "absent → present, never back" beside the existing
   "absence is unknown, not violated".
5. `src/change_tests.rs` — three discriminators and one control, next to the #711 tests they
   extend. Verify the discriminators fail with the fix disabled in place and the control passes
   both ways.
6. `specs/change/*` — invariant, contract clause, error case, requirement, tasks, testing notes,
   and the module's own lesson in `context.md`.
7. `cargo clippy -- -D warnings` **bare** (this is what CI runs; `change check` does not run
   clippy at all), full `cargo test`, then `change check` → `change audit --strict`.

## Delivery scope (frozen at `change new`)

- `src/change.rs`
- `src/change_tests.rs`
- `specs/change/change.spec.md`
- `specs/change/requirements.md`
- `specs/change/context.md`
- `specs/change/tasks.md`
- `specs/change/testing.md`
