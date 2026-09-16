---
hi: 1
families: [UPKEEP]
---

# Keeping the record readable

## Intent

Documents that live for years accumulate: change logs grow, finished tasks pile up, two branches edit the same table. None of that should be a reason to stop keeping the record. Trimming, archiving and untangling should each be one command, with a preview when something is about to be thrown away.

## Criteria

- **UPKEEP-1**  A contract's change log can be trimmed back to its recent entries so it does not grow without bound.
  - **UPKEEP-1.a**  I can see what would be dropped before anything is written.
- **UPKEEP-2**  Finished tasks move out of the way into an archive section, so what is left on the page is what is still live.
- **UPKEEP-3**  When two branches edit the same contract, the merge conflict can be resolved with an understanding of the file's structure.
  - **UPKEEP-3.a**  A conflict left behind in a contract is found even when git no longer reports it.
- **UPKEEP-4**  After pulling or switching branches, I can make the next check look at everything again.
