---
change: CHG-0128-every-command-that-derives-a-module-s-api-must-honour-the-configured-export-leve
artifact: testing
---

# Testing

Sandbox gate 062 is the judge, run against a build of `origin/main` and against
this one:

    before (origin/main 48caf568)   pass=4 fail=0 pending=1   FAIL
    after  (this change)            pass=5 fail=0 pending=0   PASS

Full board, to prove nothing else moved:

    pass=38 fail=17 skip=0 total=55

Exactly one drill changed state. That check exists because the previous batch
shipped a fix which closed its own gate and created a fresh defect elsewhere,
invisible to a per-gate run.

Healthy controls inside gate 062, still green after the fix — the fix narrows
the surface to what was configured, it does not stop counting:

    check --strict, export_level=type     rc=0, 2/2 documented
    check,          export_level=member   rc=1, still errors on id/name/find
    score,          export_level=member   api 8, still names id/name/find

Hand-measured consequences beyond `score`, before and after, same fixture:

    new       before: Public API rows Profile, id, name, ProfileCatalog, find
                      activate + check -> rc=1 "Spec documents 'id' but no
                      matching export found in source"
              after:  Profile, ProfileCatalog; activate + check -> no errors
    diff      before: "new_exports": ["id","name","find"]
              after:  "new_exports": []

Dogfood: `check --root <worktree>` rc=0, 62 specs passed, 105/105 coverage.

Suite: fmt clean, clippy clean, 2244 unit + 367 integration, 0 failures.

## Requirement evidence

| Requirement | Evidence |
|-------------|----------|
| REQ-scoring-004 | Gate 062 flips 1 pending to 0. `scoring::tests::api_score_grades_against_configured_export_level` fails on unfixed code. The member-level control still deducts, so the surface was narrowed rather than the counting disabled |
| REQ-generator-004 | A generated spec, activated and checked, produces no orphan-export errors where it previously produced three. This is the consequence nobody reported: the tool was generating work its own validator rejected |
| REQ-exports-008 | Both wrappers retained with `#[allow(dead_code)]` and a doc comment naming #474. Deleting them was tried and reverted — this repo's own exports spec documents them, so removal failed spec-sync's own drift check. The guard is weaker than deletion and that is stated in the design rather than glossed |
| REQ-cmd-new-003 | Measured above: the Public API it writes no longer contains symbols `check` rejects |
| REQ-cmd-scaffold-003 | Same path, same fixture |
| REQ-cmd-diff-003 | `new_exports` goes from three false entries to none; drill 043 (parse-mode agreement, 200 assertions) is green before and after, which matters because parse_mode is now threaded here too |
