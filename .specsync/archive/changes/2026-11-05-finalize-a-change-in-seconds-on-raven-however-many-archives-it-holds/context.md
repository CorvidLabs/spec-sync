---
change: finalize-a-change-in-seconds-on-raven-however-many-archives-it-holds
artifact: context
---

# Context

Leif asked in Grok Bot chat t813u (2026-11-03) for SpecSync changes and archive and everything to work fast, after reports that "sometimes SpecSync archive takes like three minutes to run" (hi SPEED-2). This stacks on REQ-change-119 (#840), which made archived reads fast but left mutating commands reading live.

Profiled on Linux against a scratch branch of CorvidLabs/raven (127 archived changes): one `change finalize` with the #840 build took 50.1 s quiet and 101 s under load, spawning 10,948 Git processes and spending 53.6 s in Git. Almost all of it was the archive walk finalize runs to prove the new archive fits: every other archive's workflow-version history and workflow-v2 baseline history re-derived from scratch, plus 2,170 `rev-parse --show-prefix` calls.
