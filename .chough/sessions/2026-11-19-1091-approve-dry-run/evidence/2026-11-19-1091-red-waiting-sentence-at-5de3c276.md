---
kind: evidence
id: 2026-11-19-1091-red-waiting-sentence-at-5de3c276
aliases: []
claim: "Red-first, for ss-lead's finding on e24bdbf9: at 5de3c276 (the test alone) approve_dry_run::the_dry_run_refuses_while_an_interrupted_write_is_waiting fails on the exact sentence. e24bdbf9 had rewritten the message as one source line without its line continuations, so it printed runs of spaces ('the next          lifecycle command'). My own regression from this session, fixed in 2d8abc60; not a new bug."
result: failed
evidence_kind: product-test
revision: fd02224d3f77af1d9c40b5a4b44817f0cc6d9d13
content: 3acfd6aa51ad4a0f77535ba33461f8fb6af9d6c87ee7c7ffbd06cf457d96d505
environment: macOS 27.2 (Darwin 27.2.0), Apple M5 Ultra, local clone
fingerprint: sha256:77e2c8370e266b0010f06eadc019544cb519eb5f7d14098bf65f9c6fd564fde1
external: ext-c4ab44
criteria: [CHANGE-15.a]
requirements: [REQ-change-163]
contracts: [specs/change/change.spec.md]
limitations: []
observed: 2026-11-19T07:52:41Z
actor:
  - agent: claude-code
    terminal: t59
---
**Red-first, for ss-lead's finding on e24bdbf9: at 5de3c276 (the test alone) approve_dry_run::the_dry_run_refuses_while_an_interrupted_write_is_waiting fails on the exact sentence. e24bdbf9 had rewritten the message as one source line without its line continuations, so it printed runs of spaces ('the next          lifecycle command'). My own regression from this session, fixed in 2d8abc60; not a new bug.**

Recorded as failed in macOS 27.2 (Darwin 27.2.0), Apple M5 Ultra, local clone at revision fd02224d3f77, observed 2026-11-19T07:52:41Z.
