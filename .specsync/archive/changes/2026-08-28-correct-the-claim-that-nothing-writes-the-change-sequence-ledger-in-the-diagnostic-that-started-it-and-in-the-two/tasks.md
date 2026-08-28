---
change: correct-the-claim-that-nothing-writes-the-change-sequence-ledger-in-the-diagnostic-that-started-it-and-in-the-two
artifact: tasks
---

# Tasks

- [x] Correct the diagnostic at `src/change.rs` to state the true constraint (nothing allocates) rather than the false one (nothing writes)
- [x] Correct `AGENTS.md`, which asserted "Nothing writes it" and then described the write in the same bullet
- [x] Correct the `CHANGELOG.md` entry for #665, and say in the entry that it carried the wrong claim until it was measured
- [x] Correct Discussion #339, the public roadmap, stating the correction rather than editing silently
- [x] Confirm no test pinned the old wording
- [x] Confirm no other live file carries the claim, and leave `specs/change/context.md` to the concurrent #714 audit that owns it
- [x] Record the lesson that an error message is untested prose, and the lesson that `--spec` and `--no-spec-change` compose
