---
change: stop-the-archive-check-and-change-pr-from-telling-a-copy-already-archived-upstream-to-run-ship
artifact: context
---

# Context

Issue #902, a follow-up from the #889 review (thread 4). #889 made `change review`, `finalize`, `status` and `ship-status` refuse or report a change whose id the remote default already archived while this branch holds an unarchived copy (`upstream_archive_of`, REQ-change-129, REQ-cmd-change-029). The shared archive-candidate decision, `archive_candidate_can_ship` in `src/commands/change.rs`, and its shell twin `.github/scripts/archive-readiness.sh` were not taught that condition. Both still classify a stale copy with a passing review whose `implementation_commit` equals `HEAD` as shippable, so the pull-request sentence (`change status` without an id, `change pr`) and the Archive check can print `Run specsync change ship <id>`, a command `ship` now refuses.

The CI Archive job checks out a single commit (`actions/checkout` default depth), so no remote default ref exists there. The script cannot see upstream archives in that job; it keeps today's classification there, and the job fails on any open change regardless. Locally, where `change pr` and `ship` run, `origin` is present.

Out of scope: making the Archive job fetch the remote default (a CI cost decision), and any change to `upstream_archive_of` itself.
