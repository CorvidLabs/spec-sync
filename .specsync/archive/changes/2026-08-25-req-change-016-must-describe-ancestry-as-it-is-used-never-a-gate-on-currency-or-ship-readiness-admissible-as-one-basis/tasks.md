---
change: req-change-016-must-describe-ancestry-as-it-is-used-never-a-gate-on-currency-or-ship-readiness-admissible-as-one-basis
artifact: tasks
---

# Tasks

- [x] Re-verify the three `verification_commit_is_accepted_current` call sites against current
      `src/change.rs`; the issue's line numbers had drifted (13782/13787/14516 → 13874/13879/14608)
      but the shapes are unchanged: two hard conjuncts, one of three disjuncts.
- [x] Write `deltas/change.md` as `## MODIFIED` / `### REQUIREMENT REQ-change-016`, generated from
      the live requirement body so the five untouched bullets stay byte-identical.
- [x] Replace only the `verification.commit` bullet with the scoped claim plus the MUST NOT
      obligation.
- [x] Approve, then `change check` to materialize the delta into `specs/change/requirements.md`.
- [x] Assert the materialized requirement body is byte-identical to the delta section body.
- [x] `change audit --strict`.
