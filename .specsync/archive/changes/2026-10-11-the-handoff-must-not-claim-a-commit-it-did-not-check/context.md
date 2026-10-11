---
change: the-handoff-must-not-claim-a-commit-it-did-not-check
artifact: context
---

# Context

A dogfood of the split on main approved a no-spec change, ran `change check` without `--commit`, and left the change directory untracked. The handoff still said the implementation was committed and a fresh session could resume at review. A new clone of HEAD would not contain that change.

`HandoffSignals` records whether affected paths are dirty and whether verification matches this working tree. It does not record that a commit happened. Evidence under `.specsync/` stays uncommitted until finalize on purpose. The reason may claim only those signals.
