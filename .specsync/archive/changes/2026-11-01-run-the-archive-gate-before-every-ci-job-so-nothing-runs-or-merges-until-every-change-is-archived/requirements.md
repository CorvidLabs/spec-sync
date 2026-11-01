---
change: run-the-archive-gate-before-every-ci-job-so-nothing-runs-or-merges-until-every-change-is-archived
artifact: requirements
---

# Requirements

### REQ-github-023

CI SHALL be archive-first. The `Archive` job SHALL run before every other job, and every other job
SHALL depend on it, so a pull request tip with any open change under `.specsync/changes/` runs
nothing else. The full lane SHALL run on the archived tip itself; no tip SHALL reuse trust from a
green product parent.

Acceptance Criteria
- A tip with an open change fails `Archive`, every other job is skipped, and the required CI gate
  fails.
- A tip with every change archived passes `Archive` and runs the lane classify selects from the
  whole pull request, not from the tip commit alone.
- `is_trust_relevant_check_name("Archive")` is true, so ship-status lists the Archive check among
  trust-relevant checks. A name that only contains the word, such as `Archive old releases`, is not
  matched.

### REQ-cmd-change-007 (modified)

Ship-status trust guidance and stage actions describe archive-first CI: verify and review locally, ship, push the archive tip. Live check-runs are read on HEAD first. Full text in `deltas/cmd_change.md`.

### REQ-cmd-change-019 (modified)

The Archive check runs before every other CI job. While it fails no other job runs, and both SpecSync implementation ready and the required CI gate fail. Full text in `deltas/cmd_change.md`.
