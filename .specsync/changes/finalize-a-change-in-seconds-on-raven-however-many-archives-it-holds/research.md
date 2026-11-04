---
change: finalize-a-change-in-seconds-on-raven-however-many-archives-it-holds
artifact: research
---

# Research

A logging Git shim on the Raven scratch branch (Linux, 8 cores, load 6 to 12): 10,948 Git processes for one finalize with the #840 build; 6,427 once finalize reuses the persistent history verdicts; 2,639 once Git reads that name only full commit ids are memoized; 1,274 once the workflow-v2 baseline history verdict is memoized per resolved `HEAD` id. What remains per archive is a live `rev-parse --verify HEAD` and that archive's own committed state. Finalize and archive never move a ref themselves: the archive commit is made by the CLI after `finalize_change` returns, and no Git call under `src/change` commits, checks out, resets or updates a ref.
