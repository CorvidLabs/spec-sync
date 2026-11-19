---
change: change-correct-reads-its-summary-inside-a-change-read-scope
artifact: context
---

# Context

`change correct` (`correct_interview_metadata`) returns the corrected change's summary. It built
that summary with only the project lock's fact scope, no change read scope, so it got none of
the memoized reads, persistent read cache or prefetch workers that `change status`, `show`,
`list` and `ship-status` use. On a Raven-sized fixture one unscoped summary took 158 s, against
34 s cold and 1.5 s warm inside a scope (#921).

The read scope's own contract: mutating lifecycle APIs keep their validation live, and a scope
must not be held across the command's own writes. So only the summary, taken after the writes,
moves into a scope.
