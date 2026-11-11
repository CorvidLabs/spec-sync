---
change: approve-accepts-a-sent-back-change-s-revised-body-for-a-requirement-it-added-itself
artifact: context
---

# Context

Issue #1001, found while re-scoping #994 after a send-back. Materialization
(`prepare_module_application`) treats a sent-back change's own earlier ADDED block as
MODIFIED when `sent_back_added_block_is_own` finds the send-back listed it as applied
and the canonical tree still carries that text (#914/#960). `change approve` runs
`validate_delta_files` first, which refused any ADDED requirement whose living text
differs ("cannot add existing block"), with no such exception. A sent-back change
therefore could not be re-approved with a revised body for a requirement it added
itself. The #994 workaround deleted the materialized blocks by hand.

`validate_delta_files` now asks the same helper before refusing, reading the same
ledger and canonical files materialization reads. A block someone else changed is
still refused.

Authorization: Leif's 2026-11-10 merge rule lets bug fixes that make behavior match
existing requirements (here REQ-change-123's send-back criterion) run the full lifecycle
with 0xLeif as approve and review actor.
