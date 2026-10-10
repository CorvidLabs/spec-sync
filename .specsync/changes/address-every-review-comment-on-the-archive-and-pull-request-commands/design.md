---
change: address-every-review-comment-on-the-archive-and-pull-request-commands
artifact: design
---

# Design

`change pr` opens or reuses an open pull request even when the change is not ready to archive, and it archives only when this change is ready and first. It runs the pre-push gate before pushing and prints one result. Merge guidance is the archive sentence: merge only when nothing is open.

The Archive check and the Rust ranking share one lightweight proof for recommending ship: workflow v2, verdict pass, and the reviewed commit equal to HEAD. Accepted ranks ahead of verifying. An archived state left under `.specsync/changes/` stays open.

Archive stays out of SpecSync implementation ready and stays on the required merge gate.
