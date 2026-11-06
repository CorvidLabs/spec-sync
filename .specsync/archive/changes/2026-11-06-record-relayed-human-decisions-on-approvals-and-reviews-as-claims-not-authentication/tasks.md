---
change: record-relayed-human-decisions-on-approvals-and-reviews-as-claims-not-authentication
artifact: tasks
---

# Tasks

- [x] Capture CHANGE-6.d, CHANGE-13 to 13.d and SHIP-2.e in hi, retire CHANGE-6.b.
- [x] Write failing tests first.
- [x] Add `RelayedDecision` with read-side validation and the flag builder.
- [x] Record the relay on definition approvals, including the portable pair.
- [x] Record the relay on scoped reviews and leave it out of the review digest.
- [x] Show recorded claims and relays in `change status` text and JSON.
- [x] Update CLI help, README, the workflow and CLI guides, and the CHANGELOG.
- [x] Record requirement evidence in the module testing files.
- [x] Codex review of #900: keep `--verdict` in the named re-run, leave relays out of correction digests, and check the decider on read; file #904 for the uncommitted-review recovery.
- [x] Second Codex round on #900: validate relay fields in the write APIs, check every review attempt's decider, ignore relays in review-history comparisons.
