---
change: keep-the-upstream-archive-status-guidance-behind-a-sequence-ledger-freeze
artifact: testing
---

# Testing

`tests/integration/archived_upstream.rs::a_sequence_ledger_freeze_outranks_the_upstream_archive_guidance`: the auditor's squash fixture with `origin/main` holding the archive, then an acknowledgement in `.specsync/change-sequence.json` that freezes the ledger. Asserts JSON `next_action` and the first `handoff.before_clearing` entry name the sequence ledger, not the upstream archive, the text `Next:` line names the sequence ledger, and `change ship-status` names the freeze before the upstream archive. On main cd8cab5e the handoff assertion fails.

The existing `archived_upstream` tests (no freeze) stay green, covering the unchanged guidance.

## Requirement evidence

| Requirement | Evidence |
| --- | --- |
| REQ-change-138 | `a_sequence_ledger_freeze_outranks_the_upstream_archive_guidance` (JSON `next_action` and `handoff.before_clearing` name the freeze), `a_change_already_archived_on_the_remote_default_is_named_and_refused_before_acceptance` (without a freeze both name the upstream archive) |
| REQ-cmd-change-034 | `a_sequence_ledger_freeze_outranks_the_upstream_archive_guidance` (text `Next:` and `change ship-status` name the freeze first) |
