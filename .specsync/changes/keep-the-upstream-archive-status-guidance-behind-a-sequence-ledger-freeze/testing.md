---
change: keep-the-upstream-archive-status-guidance-behind-a-sequence-ledger-freeze
artifact: testing
---

# Testing

`tests/integration/archived_upstream.rs::a_sequence_ledger_freeze_outranks_the_upstream_archive_guidance`: the auditor's squash fixture with `origin/main` holding the archive, then an acknowledgement in `.specsync/change-sequence.json` that freezes the ledger. Asserts JSON `next_action` and the first `handoff.before_clearing` entry name the sequence ledger, not the upstream archive, and the text `Next:` line names the sequence ledger. On main cd8cab5e the handoff assertion fails.

The existing `archived_upstream` tests (no freeze) stay green, covering the unchanged guidance.
