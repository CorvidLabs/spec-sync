---
change: the-corrupt-archive-recovery-step-names-summary-terminal-evidence-reason-the-field-change-status-json-actually-prints
artifact: context
---

# Context

Issue #966, from the Codex review of #901 (P2, "Point recovery guidance at the emitted JSON field"), deferred by Leif's call.

When an archive fails its integrity check but its package still matches the commit that archived it (`CorruptArchive::OutsidePackage`), the recovery step says to run `specsync change status '<id>' --json` and restore what its `terminal_evidence.reason` names. `change status --json` nests the change summary under `summary` (`print_record`), so that top-level field does not exist and the step cannot be followed as written.

HI: CHANGE-7.a.2, "I am told the exact command that restores that archive from trusted history, and once it is restored I am told it is safe again."
