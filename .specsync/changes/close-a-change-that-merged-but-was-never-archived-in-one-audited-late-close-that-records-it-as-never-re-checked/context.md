---
change: close-a-change-that-merged-but-was-never-archived-in-one-audited-late-close-that-records-it-as-never-re-checked
artifact: context
---

# Context

#833: archived changes left in state `accepted` have no supported repair, and one of them blocks every later
`change finalize` in the repository. Corvidinho has 46 such packages. On CorvidLabs/site, 38 changes merged
long ago were never archived, and CHG-0026 (site PR #232) blocks every finalize. Its documented recovery,
`change reopen`, fails before it checks anything on a bounded Git read over 240 exact-delivery entries
(#976, fixed separately). About 70 of those entries drifted, and one is `.specsync/change-sequence.json`,
which nothing can own. 15 of the 38 are legacy v1 records, and a probe saw `change accept` refuse one after
a passing `check`. Site #616 sits in the partial-finalize state #833 describes (state `accepted`, a written
`finalization.json`, still under `.specsync/changes/`).

Intent: SHIP-12 and SHIP-12.a to SHIP-12.f, captured verbatim in hi/ship.md from Leif's "Capture all +
approve design" (relayed by orc t18 on 2026-11-06). The same message decided the design:

- D1: `closed_late` is a new terminal state.
- D2: drafts can be closed late and are recorded as never approved.
- D3: several IDs in one all-or-nothing call, plus `--dry-run`.
- D4: this PR goes first, and the reopen overflow (#976) follows as its own PR.

Requirement ID REQ-change-143 was claimed by the orc.
