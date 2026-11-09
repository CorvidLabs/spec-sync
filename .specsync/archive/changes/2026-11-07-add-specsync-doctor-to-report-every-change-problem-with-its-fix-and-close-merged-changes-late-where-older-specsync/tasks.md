---
change: add-specsync-doctor-to-report-every-change-problem-with-its-fix-and-close-merged-changes-late-where-older-specsync
artifact: tasks
---

# Tasks

- [x] Write the failing tests and record that they fail on 1276e314.
- [x] Add the closed-late location, `late-close.json`, the 6.3 lookup, the refusals and the slug reservation.
- [x] Add `specsync doctor`: the scan, the classes, `--fix`, `--dry-run`, `--close-late`, `--actor` and `--reason`.
- [x] Add the compatibility script with 6.0.0 and 6.2.0 pinned by sha256, the CI job and the fledge lane.
- [x] Update the contracts, docs, AGENTS.md quick reference and CHANGELOG.
- [x] Materialize with change check --commit, pass audit --strict, the verify lane and pre-push.

Human gates (not tasks): scope approval and implementation review, recorded by Leif through the orc.
