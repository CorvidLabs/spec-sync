---
change: finalize-ship-and-archive-refuse-an-approvals-ledger-that-git-history-does-not-hold
artifact: research
---

# Research

A spike on a throwaway clone of main 3bebe8b0 ran the unfinished finalize check of `62d3cdc4` with an
archive-side check and the accepted-change exception added. Of 3,080 unit tests, four failed; of 627
integration tests, four failed because of the check. Six finalize or archive a ledger that is not
committed, one tampers on purpose and expects another refusal, and one expects the "archive after merge"
refusal, which the new check must come after. No test showed a legitimate committed rewrite refused.
Section 7 of the approved definition lists them.

The spike also showed that the unfinished refusal's single way out, `git checkout HEAD -- <path>`, is
wrong after `change reopen` and a plain `change check`: it would drop the reopening the tool wrote. Hence
the two ways out for a working copy, commit or restore, by what differs.
