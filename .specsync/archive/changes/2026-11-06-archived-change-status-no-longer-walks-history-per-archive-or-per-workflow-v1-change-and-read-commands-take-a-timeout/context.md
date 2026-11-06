---
change: archived-change-status-no-longer-walks-history-per-archive-or-per-workflow-v1-change-and-read-commands-take-a-timeout
artifact: context
---

# Context

S9 of `_audits/95-plan.md` (performance, with latency budgets and actionable timeout recovery). #854 and #855 were closed by #864; #831 asks for a cheap authenticated-archive read. Leif's intent SPEED-1.a is that an archived change's status returns in under 2 s on Raven, and SPEED-1.d asks for a Raven-sized benchmark.

Leif approved step 1 (measurement only) at 21:43 MST and this approach, the budgets and the timeout design at 22:35 MST on 2026-11-05, relayed by orc t18.

Leif widened the scope on 2026-11-05 at 23:24 MST, relayed by orc t18 ("Widen (C)"): prefetch workers open a same-root fact scope seeded with the parent's facts, are read-only, get no batch reader, and keep a per-worker object memo, with a cold guard. Specs git_utils and change. The send-back was recorded by the orc as 0xLeif.
