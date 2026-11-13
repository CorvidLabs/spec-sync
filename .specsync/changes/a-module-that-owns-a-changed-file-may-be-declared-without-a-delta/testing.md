---
change: a-module-that-owns-a-changed-file-may-be-declared-without-a-delta
artifact: testing
---

# Testing

src/change_tests/send_back_for_rescope.rs:

- `a_module_that_owns_a_changed_file_may_be_declared_without_a_delta`: `auth` and `sessions`
  declared over `src/auth.rs`, a delta only for `sessions`; approve, implement, materialize,
  verify and accept succeed, `sessions` carries the delta, `auth`'s spec bytes are unchanged.
  Red on main: approval refuses `missing: auth`.
- `a_module_with_no_delta_that_owns_none_of_the_files_is_refused`: the same scope over
  `src/session.rs` only is refused at approval naming `auth` and the delta rule.
- `an_approved_delta_deleted_later_is_still_a_changed_delta`: both modules approved with deltas;
  deleting `auth`'s delta afterwards is refused at materialization as changed after approval,
  although `auth` owns the declared file.
- The full unit suite stays green; fmt and clippy clean.

## Requirement evidence

| Requirement | Evidence |
|-------------|----------|
| `REQ-change-161` | `a_module_that_owns_a_changed_file_may_be_declared_without_a_delta`, `a_module_with_no_delta_that_owns_none_of_the_files_is_refused` in `src/change_tests/send_back_for_rescope.rs`, `an_approved_delta_deleted_later_is_still_a_changed_delta` |
