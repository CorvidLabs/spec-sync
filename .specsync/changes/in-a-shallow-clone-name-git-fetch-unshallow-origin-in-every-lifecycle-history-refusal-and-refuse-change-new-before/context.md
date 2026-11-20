---
change: in-a-shallow-clone-name-git-fetch-unshallow-origin-in-every-lifecycle-history-refusal-and-refuse-change-new-before
artifact: context
---

# Context

Issue #1072. In a shallow clone of an adopted project, `change new` exits 0 and writes a change that no other command can use: `change status`, `check` and `audit` refuse with `workflow-v2 baseline cutoff must be an ancestor of its introduction's first parent`, and a history failure under the correction ledger reads as `correction ledger integrity is invalid`. After `git fetch --unshallow origin` the same change reads normally (measured on spec-sync, section 3 of the definition).

Definition: ss-lead/gates/git-environments-909-1072-1077-definition.md, section 3 and REQ-change-173. Leif's choices 3 and 4 in the tester's dialog of 2026-11-20 at about 00:45Z: `change new` refuses before writing, and every such refusal in a shallow clone names the fetch.
