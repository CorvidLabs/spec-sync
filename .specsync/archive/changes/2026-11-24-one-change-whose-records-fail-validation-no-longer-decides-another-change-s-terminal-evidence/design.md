---
change: one-change-whose-records-fail-validation-no-longer-decides-another-change-s-terminal-evidence
artifact: design
---

# Design

Each verdict reads only the records it depends on (REQ-change-119, REQ-change-012, REQ-change-024):

| Change being judged | What its verdict reads | A failure there |
|---|---|---|
| Archived | its own package and its own history | its own verdict, as today |
| Accepted, every delivery input unchanged | its own package and history | its own verdict, as today |
| Accepted, an input changed | the above, and every candidate successor for that input | counts: the change is `stale` and the reason names the candidate and its failure |
| Retired | its retirement receipt and its replacement | its own refusal, as today |

The roster is loaded one change at a time: a change whose records fail validation keeps its own
failure, and every other change is judged with the changes that loaded. When a successor search for
a changed input finds no cover, a candidate that failed to load is named with its failure: a
change whose loaded record declares the input, or one whose `state.json` does not read at all. An
archived change, and an accepted change with unchanged inputs, never search.

`check`, `review`, `finalize`, `ship` and `archive` load the roster themselves and are not changed.
