---
change: CHG-0085-resolve-canonical-ownership-at-approve-and-free-never-closed-changes
artifact: plan
---

# Plan

1. Failing test: approve accepts a path owned by an undeclared module.
2. Add `validate_declared_path_ownership`; call it from approve.
3. Failing test: a never-closed verifying change cannot correct an owner.
4. Return `Option` from the reopen lookup; check the definition approval instead.
5. Scope both to cases the checks can resolve, guided by the suites.
