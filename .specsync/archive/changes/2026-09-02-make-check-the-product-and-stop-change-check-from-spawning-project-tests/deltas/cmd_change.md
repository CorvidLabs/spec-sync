## MODIFIED

### SPEC SECTION Invariants

1. JSON output contains no terminal coloring.
2. Domain errors always produce exit code 1.
3. `change check` runs scoped verification for one change only: evidence completeness then in-process spec↔code sync. It does not spawn project tests or rewalk archived terminal evidence.
4. `change audit` reports active-workspace and living-spec integrity only and exits non-zero on report errors.
5. `change finalize` requires current verification and scoped-review evidence and performs no provider merge.
6. `change ship-status` decides readiness from evidence CURRENCY — the recorded plan and tree still match what was verified — never from whether the recorded commit is reachable from HEAD. A squash-merge rewrites that commit, so reachability would make a squash-merged change permanently unfinalizable while its evidence is intact. The rule covers the scoped review as well as the verification: readiness asks whether the recorded review is current, reports that answer as `current`, `stale`, or `unavailable`, and treats only `current` as satisfied. An unavailable guarantee reported as a satisfied one is worse than the refusal it conceals, and readiness that never asks receives no negative answer and reads its own silence as a pass.
7. The lessons loop surfaces at each of the three moments a lesson exists: `change new` names every affected module's `specs/<module>/context.md` that holds substantive prose, a FAILED `change check` names where to record what the failure taught, and BOTH `change finalize` and `change ship` name folding the archived bundle into those specs before their remaining guidance. Every surface is a pointer, never a dump, and none can fail a lifecycle command. A passing `change check` says nothing, and a change owning no affected specs receives the same guidance it received before the fold-back existed. Both verbs also emit a `lesson_bundle` path in `--json`.
