## ADDED

### REQUIREMENT REQ-change-173

When a check of the project's lifecycle history refuses in a shallow clone (`git rev-parse --is-shallow-repository` is `true`), SpecSync SHALL say the clone is shallow and name `git fetch --unshallow origin` instead of reporting the history or the correction ledger as invalid, and `change new` SHALL make that check before it writes anything, so a refused `change new` leaves no change behind.

Acceptance Criteria
- In a depth-1 clone and a depth-50 clone of an adopted project, `change new`, `change status`, `change check` and `change audit` refuse, saying the clone is shallow and naming `git fetch --unshallow origin`; after that fetch, the same commands behave as in a full clone.
- A refused `change new` leaves no `.specsync/changes/<id>/`.
- In a full clone, a baseline whose cutoff is not an ancestor keeps today's message, and a corrupt correction ledger keeps its own.
- The tester's two shapes (a depth-1 clone with no origin ref; a shallow clone with `origin/main` but not the merge-base) name the fetch.
