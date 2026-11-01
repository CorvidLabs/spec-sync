---
change: prepare-specsync-6-1-2-package-metadata-and-release-handoff
artifact: tasks
---

# Tasks

- [x] Inspect package surfaces and release workflows.
- [x] Prepare a draft definition without fabricated approval.
- [x] Obtain actual definition approval (0xLeif, 26a52c35).
- [x] Merge origin/main 709ab2bd and list every user-visible change since v6.1.1.
- [x] Run the three release validators and cargo package --list.
- [x] Confirm #830 is on main (59c323f4) and merged here.

Human gates and post-verification steps (not tasks): native verification with
`change check --commit`; the human implementation review; finalization on this pull
request before merge; qualification of the immutable candidate on the required hosted
platforms; and publishing and independently verifying distributions, each only after
separate authorization (see docs/releases/6.1.2.md).
