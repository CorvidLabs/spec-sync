---
change: CHG-0122-cover-the-integration-tests-added-for-the-coverage-unmeasured-matrix-which-asse
artifact: context
---

# Context

CHG-0121 declared the source paths its fix touched and the CHANGELOG, but not
the three test files it adds. The lifecycle gate caught that on the PR:

    error: meaningful changed paths are not covered by an active change:
      tests/integration.rs, tests/integration/check.rs,
      tests/integration/coverage_unmeasured.rs

Delivery scope freezes at the interview (#542), so CHG-0121 cannot be widened
to include them. This change covers exactly those three paths and nothing else.

It carries `--no-spec-change` because the files assert requirements CHG-0121
already wrote. No module's specified behaviour changes here; if it did, this
would be the wrong instrument and the requirement would belong in CHG-0121.

Worth recording for the next author: a change that adds tests must declare its
test paths at `change new`. This is the second time in this campaign that a
declared-path list was discovered to be short only after the work was finished —
CHG-0119 named the wrong owning module for `src/commands/comment.rs` and had to
be re-cut as CHG-0120. Both were caught by a gate rather than by review, which
is the system working, but both cost a redo that a moment's checking at
interview time would have avoided.
