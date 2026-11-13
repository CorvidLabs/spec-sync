---
kind: session
id: 2026-11-13-493-ci-guide-on-the-site
aliases: [493-ci-guide-on-the-site]
objective: Publish the CI confidence guide on the site and test that each expensive suite has one owner (#493)
intent_scope: ""
modules: [cmd_change]
stage: accept
stage_status: active
iteration: 1
callers: []
native_change: publish-the-ci-confidence-guide-on-the-site-and-test-that-each-suite-has-one-owner
pr: ""
criteria: [DOCS-2]
requirements: []
cases: []
design: []
files:
  - site/src/content/docs/ci-confidence.md
  - docs/ci-confidence.md
  - .github/scripts/test-ci-lane-ownership.py
  - .github/workflows/ci.yml
  - fledge.toml
observations:
  - 2026-11-13-at-85e7fa652-docs-493-with-origin-main-bb7682a78-merged-in
  - 2026-11-13-red-first-control-with-the-ownership-test-and-no-site-page
bugs: []
blockers: []
waiting_on_human:
  - gate: Implementation review of publish-the-ci-confidence-guide-on-the-site-and-test-that-each-suite-has-one-owner
    holds: accept
    drives: review
    opened: 2026-11-13T20:08:07Z
run_consents: []
next: Read scoped intent and native status, then choose the next action within the authorized scope.
---
# Publish the CI confidence guide on the site and test that each expensive suite has one owner (#493)

## Goal

Publish the CI confidence guide on the site and test that each expensive suite has one owner (#493)

## Where it stands

- Merged main, ran the full verify lane on the merged tree, recorded the red control

## Decisions and why

- decided: authorization is Leif's own approval ('#493 CI guide on the site', feature set 2, coordinator chat 2026-11-13T03:49Z; go-ahead as designed 04:04Z; relayed by claude-code-t370), recorded natively as 0xLeif in relay form
- decided: no new lesson, because the change is narrow and an existing lesson covers it

## What was verified

- [At 85e7fa652 (docs/493 with origin/main bb7682a78 merged in), fledge lanes run verify passed 10/10 steps (unit 3070 passed, 2 ignored; integration 619 passed; strict spec check; CI script tests), and fledge run ci-lane-ownership-test passed: the trust-lifecycle lane runs no test, lint or verify step, the verify lane keeps fmt, lint, types, test, build and spec-check, and the guide is published at site/src/content/docs/ci-confidence.md.](evidence/2026-11-13-at-85e7fa652-docs-493-with-origin-main-bb7682a78-merged-in.md)
- [Red-first control: with the ownership test and no site page, LaneOwnershipTests.test_the_guide_is_published_on_the_site fails. The page this change publishes is the fix; not a new bug.](evidence/2026-11-13-red-first-control-with-the-ownership-test-and-no-site-page.md)

## Open threads

## Timeline

- 2026-11-13T19:23:32Z · start · intent → intent · Publish the CI confidence guide on the site and test that each expensive suite has one owner (#493) ^t-5c71dc
  - revision: 85e7fa652f939c530919a7d0be61e2c67c08a79c
  - by: claude-code (hc t38p)
- 2026-11-13T19:23:34Z · advance · intent → design ·  ^t-4a2434
  - native_change: publish-the-ci-confidence-guide-on-the-site-and-test-that-each-suite-has-one-owner
  - revision: 85e7fa652f939c530919a7d0be61e2c67c08a79c
  - by: claude-code (hc t38p)
- 2026-11-13T19:23:34Z · skip · design → plan · CLI, CI and docs behaviour only: no visuals or interactions ^t-7e2797
  - native_change: publish-the-ci-confidence-guide-on-the-site-and-test-that-each-suite-has-one-owner
  - revision: 85e7fa652f939c530919a7d0be61e2c67c08a79c
  - by: claude-code (hc t38p)
- 2026-11-13T19:23:36Z · advance · plan → build · native:verifying ^t-371a10
  - native_change: publish-the-ci-confidence-guide-on-the-site-and-test-that-each-suite-has-one-owner
  - gate: native:verifying
  - revision: 85e7fa652f939c530919a7d0be61e2c67c08a79c
  - by: claude-code (hc t38p)
- 2026-11-13T19:23:36Z · advance · build → test · native:verifying ^t-637716
  - native_change: publish-the-ci-confidence-guide-on-the-site-and-test-that-each-suite-has-one-owner
  - gate: native:verifying
  - revision: 85e7fa652f939c530919a7d0be61e2c67c08a79c
  - by: claude-code (hc t38p)
- 2026-11-13T20:08:07Z · advance · test → accept · native:verifying ^t-6cb33d
  - native_change: publish-the-ci-confidence-guide-on-the-site-and-test-that-each-suite-has-one-owner
  - gate: native:verifying
  - revision: 85e7fa652f939c530919a7d0be61e2c67c08a79c
  - by: claude-code (hc t38p)
- 2026-11-13T20:08:07Z · gate-opened · accept → accept · Implementation review of publish-the-ci-confidence-guide-on-the-site-and-test-that-each-suite-has-one-owner ^t-273332
  - native_change: publish-the-ci-confidence-guide-on-the-site-and-test-that-each-suite-has-one-owner
  - revision: 85e7fa652f939c530919a7d0be61e2c67c08a79c
  - holds: accept
  - drives: review
  - by: claude-code (hc t38p)
