---
hi: 1
families: [SPEED]
---

# Speed

## Intent

Captured from Leif's ask in Grok Bot chat t811u on 2026-11-03: "Leif wants the workflow quick: no command should keep an agent waiting minutes, and nothing should push agents to work around Chough." The criteria below are his "Want" for SpecSync speed, in his words.

## Criteria

- **SPEED-1**  No command should keep an agent waiting minutes.
  - **SPEED-1.a**  Status of an archived change, `change list` and `change status` return in under 2 seconds on Raven.
  - **SPEED-1.b**  `check` stays proportional to the specs touched.
  - **SPEED-1.c**  A cache keyed by content digest has a correct invalidation rule.
  - **SPEED-1.d**  A benchmark against a Raven-sized fixture keeps this from regressing.
