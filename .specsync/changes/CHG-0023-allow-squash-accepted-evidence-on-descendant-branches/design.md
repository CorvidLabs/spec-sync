---
change: CHG-0023-allow-squash-accepted-evidence-on-descendant-branches
artifact: design
---

# Design

Extend the final commit-reachability condition in `ensure_closing_approval_valid`. When the verification commit is not an ancestor and the current workspace is not byte-identical to remote main, accept only if the remote default branch records the same change in accepted state. All definition, acceptance-input, verification, and closing-approval validation remains earlier and unchanged.
