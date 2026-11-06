---
change: hash-a-change-s-clean-delivery-files-one-blob-at-a-time-so-reopen-does-not-overflow-a-single-git-read-on-large-affected
artifact: context
---

# Context

#976: `specsync change reopen` refuses before it checks anything when a change's affected paths hold more
clean file bytes than one in-memory Git read allows. The refusal is "`git cat-file --batch` output exceeds
deterministic bounds". On CorvidLabs/site, reopen of CHG-0026 (site PR #232) fails this way in 1.8 s on a
throwaway clone of site origin/main 3d858b83, whose `public/nevermore` holds 609 files and 372 MiB.

Intent: SHIP-6 ("When closed evidence goes stale there is an audited way to reopen it"). No new criterion;
Leif approved the plan through orc t18 ("Capture all + approve design", 2026-11-06), and the orc claimed
REQ-change-144. The late close (#833, REQ-change-143) is a separate PR and goes first.
