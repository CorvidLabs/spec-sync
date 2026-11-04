---
change: answer-archived-change-status-change-list-and-change-status-within-2-seconds-on-raven
artifact: context
---

# Context

Leif asked in Grok Bot chat t811u (2026-11-03) for status of an archived change, `change list` and `change status` to return in under 2 seconds on Raven, with `check` staying proportional to the specs touched, caching by content digest with a correct invalidation rule, and a benchmark against a Raven-sized fixture (hi SPEED-1). On Raven (124 archived changes, 120 specs) 6.1.1 took more than 2 minutes on Apple silicon for one archived status read.

Profiled on Linux against CorvidLabs/raven main 74e43005: one archived `change status <id> --json` took 40.4 s with 6.1.1 and 40.8 s with main, spawning 9,972 Git processes. Three costs dominated: terminal evidence was computed for every accepted or archived record when only one was asked for; scoped-review Git reads were repeated without a memo; and every archive's workflow-version history and archive introduction were re-walked on every call. Active-change reads were 0.13 to 1.7 s and `check` 1.6 to 3.5 s on Linux, so they were already proportional; the minutes Leif saw there are consistent with the archived reads underneath plus macOS process cost.
