---
change: leave-relayed-decisions-out-of-approval-succession-the-definition-digest-and-the-trusted-correction-history
artifact: context
---

# Context

#900 made a relayed decision (`relayed_by`) a claim that no digest covers and no gate reads, so an older 6.x binary that drops the unknown key computes the same answers ("Keep it out", Leif, 2026-11-05). The review digest, both correction digests and the review-history comparisons already leave it out. Codex found three comparisons that still see it:

- #926: the approval-succession check of a reopened package (`ledger_succession`) compares approvals and reopen events as raw JSON, so an older binary's reopen, which rewrites `approvals.json` without the key, is not a lawful successor and finalization is blocked.
- #927: the definition digest hashes the correction ledger as written, so a relay embedded in a correction's superseded approvals is in the digest the approval binds and the execution digest reaches; editing a relay changes gate evidence, and an older binary computes a different digest.
- #971: the trusted correction-history comparison compares whole corrections, so an older binary's append reads as divergence.

Leif captured CHANGE-13.h on 2026-11-06 (relayed by orc t18, "One sentence"). The orc assigned REQ-change-relay-compare-001.

The failing tests are committed first (fe9615c1). Ruled out: keeping the raw comparisons and documenting the limit (it leaves CHANGE-13.h false), and weakening any of the three checks beyond the relay (every other field is still compared exactly).
