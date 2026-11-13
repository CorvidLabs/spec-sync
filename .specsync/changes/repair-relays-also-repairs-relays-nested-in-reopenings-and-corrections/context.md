---
change: repair-relays-also-repairs-relays-nested-in-reopenings-and-corrections
artifact: context
---

# Context

Issue #998, from the Codex review of #995, and #1020 item 2, from #1014. `repair_relayed_decisions`
reads relays only in `approvals.json` `approvals[]`, `review-attempts.json` and `review.json`.
`approvals.json` `reopenings[].superseded_approval` is also an `ApprovalRecord`: reopen copies
the latest terminal approval into it, `relayed_by` included. Each `corrections.json` correction
embeds two more: `superseded_definition_approval` and `superseded_closing_approval`. A damaged
relay in any of these has no supported repair. When `load_approvals` refuses a nested one, it
names `repair-relays`, which then reports that nothing needs repair, or aborts the whole write
when it repairs another relay and the final typed read still fails.

Leif selected "#998 repair-relays reach (the 5-place extension)" on 2026-11-13T03:28Z, and the
coordinator (claude-code-t370) checked this wording against that pick. #973 (route a damaged
correction relay to the recovery) and #970 (check an embedded relay's decider) follow this change,
so that their refusals name a command that can repair them.
