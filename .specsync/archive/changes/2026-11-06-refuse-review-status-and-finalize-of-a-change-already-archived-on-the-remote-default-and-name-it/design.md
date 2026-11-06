---
change: refuse-review-status-and-finalize-of-a-change-already-archived-on-the-remote-default-and-name-it
artifact: design
---

# Design

One check, shared: "is this change id already archived in reachable history while this record is not archived?" It reads the existing `archive_introduction_index` (which already walks `HEAD` and the remote default) and returns the reference and commit of the archive introduction.

- `change review`: run the check before recording a verdict. On a hit, refuse; nothing is written.
- `change status` / `change ship-status` / handoff: on a hit, readiness is not `safe`, the next action is "merge or rebase onto <reference>; its archive of <id> at <commit> is authoritative", never finalize or ship.
- `change finalize`: on a hit, refuse with the same message before the trusted-transition preflight. The preflight itself is not changed.

Message shape: "`<id>` is already archived on `<reference>` at `<commit>`; this branch holds an unarchived copy. Merge or rebase onto `<reference>`; do not finalize it again."
