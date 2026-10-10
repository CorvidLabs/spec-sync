---
change: make-it-obvious-on-a-pull-request-when-a-change-is-still-open-and-when-it-is-archived
artifact: design
---

# Design

The Archive job is always selected. Its script reads `.specsync/changes/*/state.json` and does not build Rust. An unreadable state file still counts as open.

The change closest to archive goes first: verifying, then approved, then implementing, then draft, then anything else, and within a state the lowest id. A verifying change that already has review and verification evidence is told to run ship for that id. Every other open change is told to run status for that id.

`change ship` commits with the archive lifecycle message after finalize. Passing `--push` still pushes that commit. It does not commit twice.
