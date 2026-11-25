---
hi: 1
families: [SPEED]
---

# Speed

## Intent

Captured from Leif's ask in Grok Bot chat t811u on 2026-11-03: "Leif wants the workflow quick: no command should keep an agent waiting minutes, and nothing should push agents to work around Chough." The criteria below are his "Want" for SpecSync speed, in his words.

Captured from Leif's follow-up in Grok Bot chat t813u on 2026-11-03: "I also want to make sure that SpecSync changes and archive and everything are working fast and all the tools are working good. I've had reports that sometimes SpecSync archive takes like three minutes to run". SPEED-2 is that ask, in his words.

Approved by Leif in the Herdr orc tab on 2026-11-05 09:21 MST (relayed by the orc, choice "Approve intent + approach") for issue #861, after v6.2.0-rc.1's macOS qualification timed out at 45 minutes: "macOS RC qualification finishes well under 45 min so the timeout returns to 45, without dropping or weakening any test". SPEED-2.b.1 and SPEED-2.b.2 are that intent. The approved approach is to cut SpecSync's git process spawns and run the suite with cargo nextest in the qualify lane.

Approved by Leif in the Herdr orc tab on the morning of 2026-11-06 (relayed by orc t18, his words "Ok yes I approve all") for issue #915, after a cold `change status <archived-id>` on a Raven-sized fixture took 34 s and 3,899 Git processes. SPEED-1.a.1 is that intent, worded as proposed and approved. The approved approach is one lifecycle-history walk for all archives, with every archive still fully validated.
Approved by Leif in the Herdr orc tab on 2026-11-06 (relayed by orc t18, his words "Capture both") for issue #939, a refusal #918's faster cutoff-tree check made when it could not read the cutoff's archive tree. SPEED-1.a.3 is that intent, worded as drafted.
Approved by Leif in the Herdr orc tab on 2026-11-06 (relayed by orc t18, his words "Capture both") for issue #940, a refusal #918's faster cutoff-tree check made for a workflow-v1 archive the history walk accepted. SPEED-1.a.2 is that intent, worded as drafted.

Said by Leif in the Claude Code release-lead chat on 2026-11-24, after v6.4.2-rc.1's macOS qualification was cancelled at its 75-minute limit with every test passed: "I don't wanna hold the release on this, but I don't wanna see it fail another time. And I don't wanna keep making it longer and longer. I want all the tests and everything to have like a max time of like 30 minutes." For issue #1161 he was shown three drafted sentences in that chat on 2026-11-25 UTC and asked which to capture or reword, and whether the first replaces SPEED-2.b.1 or sits beside it, with beside as the suggestion. His answer was "yes". SPEED-2.b.3 is his sentence. SPEED-2.b.4 and SPEED-2.b.5 are worded as drafted, from "I don't wanna keep making it longer and longer" and "I don't wanna see it fail another time". SPEED-2.b.1 stays beside them.

## Criteria

- **SPEED-1**  No command should keep an agent waiting minutes.
  - **SPEED-1.a**  Status of an archived change, `change list` and `change status` return in under 2 seconds on Raven.
    - **SPEED-1.a.1**  The first status read after a fresh clone also returns in seconds, however many archived changes the project holds.
    - **SPEED-1.a.2**  Making a read faster never turns an archived change whose history is intact into a refusal.
    - **SPEED-1.a.3**  When SpecSync cannot read the history it needs, it says it could not read it, rather than saying the change was not there.
  - **SPEED-1.b**  `check` stays proportional to the specs touched.
  - **SPEED-1.c**  A cache keyed by content digest has a correct invalidation rule.
  - **SPEED-1.d**  A benchmark against a Raven-sized fixture keeps this from regressing.
  - **SPEED-1.e**  SpecSync must never wait on a descendant's open pipe after its child exits.
- **SPEED-2**  SpecSync changes and archive and everything are working fast.
  - **SPEED-2.a**  SpecSync archive does not take like three minutes to run.
  - **SPEED-2.b**  The whole suite runs fast.
    - **SPEED-2.b.1**  macOS release-candidate qualification finishes well under 45 minutes, so its timeout returns to 45.
    - **SPEED-2.b.2**  No test is dropped or weakened to make it faster.
    - **SPEED-2.b.3**  All the tests and everything have a max time of like 30 minutes.
    - **SPEED-2.b.4**  A lane that gets slower fails at its limit; the limit is not made longer.
    - **SPEED-2.b.5**  A release candidate does not fail because a lane ran out of time.
