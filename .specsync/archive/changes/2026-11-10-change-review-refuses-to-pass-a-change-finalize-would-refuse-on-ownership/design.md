---
change: change-review-refuses-to-pass-a-change-finalize-would-refuse-on-ownership
artifact: design
---

# Design

`record_scoped_review_with_relay` already refuses a passing verdict with open tasks, inside
`if verdict == ScopedReviewVerdict::Pass`. The ownership refusal sits in the same branch,
after the tasks check and before anything is written. It calls the same
`ownership_preflight_blocker` that ship-status, ship preflight and `change status` use, so
review, ship and finalize judge ownership identically. The error is `scoped review cannot
pass: <blocker>. Recovery: <recovery>`, reusing the blocker and recovery text ship-status
prints.

Rejected: changing `archive-readiness.sh` (option a). The script runs with no binary, so it
could only stop naming `ship` altogether. Also rejected: recording an ownership verdict in
verification for the script to read (option c), which grows the evidence schema.
