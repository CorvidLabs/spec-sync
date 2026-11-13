---
change: a-module-that-owns-a-changed-file-may-be-declared-without-a-delta
artifact: delta
module: change
---

## ADDED

### REQUIREMENT REQ-change-161

A change SHALL be allowed to declare an affected module without a semantic delta when that module
owns one of the change's declared files, so a change whose requirement moved into another module
in a spec split can still declare the module that owns its code (#844, #1070). Such a code-only
module's spec SHALL NOT be edited. At least one affected module SHALL still carry a delta, and a
declared module with no delta that owns none of the change's declared files SHALL be refused at
approval, naming it.

#### Acceptance Criteria

- A change declaring `auth` and `sessions` over `src/auth.rs`, which `auth` owns, with a delta
  only for `sessions`, is approved, materialized, verified and accepted; `sessions` carries the
  delta and `auth`'s spec is unchanged.
- The same scope over a file only `sessions` owns is refused at approval, naming `auth`.
- An approved delta deleted later is still reported as changed after approval.
