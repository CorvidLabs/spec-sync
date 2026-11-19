---
kind: session
id: 2026-11-13-955-share-seeded-facts
aliases: [955-share-seeded-facts]
objective: Prefetch workers share one copy of the command's seeded repository facts instead of copying them per worker (#955)
intent_scope: ""
modules: [git_utils]
stage: accept
stage_status: active
iteration: 1
callers: []
native_change: prefetch-workers-share-one-copy-of-the-command-s-seeded-repository-facts
pr: ""
criteria: [SPEED-1.a]
requirements: []
cases: []
design: []
files: [src/git_utils.rs]
observations:
  - 2026-11-13-at-2915d230b-fix-955-with-origin-main-bb7682a78-merged-in
  - 2026-11-13-red-control-weaker-than-the-others-the-red-commit-665f8a220
  - 2026-11-19-red-control-weaker-than-the-others-as-on-11-13-on-main
  - 2026-11-19-refreshed-onto-main-3bebe8b0-with-921-942-and-978-a-repository
bugs: []
blockers: []
waiting_on_human:
  - gate: Implementation review of prefetch-workers-share-one-copy-of-the-command-s-seeded-repository-facts
    holds: accept
    drives: review
    opened: 2026-11-13T20:08:21Z
run_consents: []
next: Read scoped intent and native status, then choose the next action within the authorized scope.
---
# Prefetch workers share one copy of the command's seeded repository facts instead of copying them per worker (#955)

## Goal

Prefetch workers share one copy of the command's seeded repository facts instead of copying them per worker (#955)

## Where it stands

- Merged main, ran the full verify lane on the merged tree, recorded the red control

## Decisions and why

- decided: authorization is Leif's own scope approval ('#955 shared copy', coordinator chat 2026-11-13T04:55Z, relayed by claude-code-t370), recorded natively as 0xLeif in relay form
- decided: no new lesson, because the change is narrow and an existing lesson covers it
- Refreshed onto main 3bebe8b0 (batch 7b) under Leif's relayed answer of 2026-11-18 that the 2026-11-13 approvals stand, 'Yes, they stand' (from his planning session hc t4c, relayed through ss-lead t58); no approved sentence changed, and no review, approval or gate close was recorded

## What was verified

- [At 2915d230b (fix/955 with origin/main bb7682a78 merged in), fledge lanes run verify passed 10/10 steps: fmt, lint and check-types, cargo test (unit 3071 passed, 2 ignored; integration 619 passed), release build, strict spec check, and the CI script tests, including git_utils::tests::a_repository_fact_seed_shares_one_copy_of_its_facts_across_workers and the prefetch-worker fact tests.](evidence/2026-11-13-at-2915d230b-fix-955-with-origin-main-bb7682a78-merged-in.md)
- [Red control, weaker than the others: the red commit 665f8a220 does not compile, because a_repository_fact_seed_shares_one_copy_of_its_facts_across_workers uses the shared-copy seed API this change introduces. No behavioural failure on the old code was demonstrated; the test pins the new sharing (one Arc copy across workers).](evidence/2026-11-13-red-control-weaker-than-the-others-the-red-commit-665f8a220.md)
- [Red control, weaker than the others, as on 11-13: on main 3bebe8b0 the failing-test commit 665f8a22 does not compile, because a_repository_fact_seed_shares_one_copy_of_its_facts_across_workers calls Arc::ptr_eq on the seed's facts, which main holds as a plain HashMap. No behavioural failure on the old code was demonstrated; the test pins the new sharing.](evidence/2026-11-19-red-control-weaker-than-the-others-as-on-11-13-on-main.md)
- [Refreshed onto main 3bebe8b0 with #921, #942 and #978; a_repository_fact_seed_shares_one_copy_of_its_facts_across_workers passes. fledge lanes run verify at fde6e01b with GIT_CONFIG_GLOBAL=/dev/null (#1092): 10/10 steps, 3081 unit and 627 integration tests, strict specs 62/62, 100% file and LOC coverage; lifecycle_commands_stay_within_git_spawn_ceilings passes; change audit --strict and pre-push passed.](evidence/2026-11-19-refreshed-onto-main-3bebe8b0-with-921-942-and-978-a-repository.md)

## Open threads

## Timeline

- 2026-11-13T19:23:36Z · start · intent → intent · Prefetch workers share one copy of the command's seeded repository facts instead of copying them per worker (#955) ^t-8dbdb7
  - revision: 2915d230bb7fdfd75aebe1c9e04f29156d6470e4
  - by: claude-code (hc t38p)
- 2026-11-13T19:23:38Z · advance · intent → design ·  ^t-a0e17f
  - native_change: prefetch-workers-share-one-copy-of-the-command-s-seeded-repository-facts
  - revision: 2915d230bb7fdfd75aebe1c9e04f29156d6470e4
  - by: claude-code (hc t38p)
- 2026-11-13T19:23:38Z · skip · design → plan · CLI, CI and docs behaviour only: no visuals or interactions ^t-39a254
  - native_change: prefetch-workers-share-one-copy-of-the-command-s-seeded-repository-facts
  - revision: 2915d230bb7fdfd75aebe1c9e04f29156d6470e4
  - by: claude-code (hc t38p)
- 2026-11-13T19:23:40Z · advance · plan → build · native:verifying ^t-459b0b
  - native_change: prefetch-workers-share-one-copy-of-the-command-s-seeded-repository-facts
  - gate: native:verifying
  - revision: 2915d230bb7fdfd75aebe1c9e04f29156d6470e4
  - by: claude-code (hc t38p)
- 2026-11-13T19:23:40Z · advance · build → test · native:verifying ^t-a2bd6b
  - native_change: prefetch-workers-share-one-copy-of-the-command-s-seeded-repository-facts
  - gate: native:verifying
  - revision: 2915d230bb7fdfd75aebe1c9e04f29156d6470e4
  - by: claude-code (hc t38p)
- 2026-11-13T20:08:21Z · advance · test → accept · native:verifying ^t-25b7b7
  - native_change: prefetch-workers-share-one-copy-of-the-command-s-seeded-repository-facts
  - gate: native:verifying
  - revision: 2915d230bb7fdfd75aebe1c9e04f29156d6470e4
  - by: claude-code (hc t38p)
- 2026-11-13T20:08:21Z · gate-opened · accept → accept · Implementation review of prefetch-workers-share-one-copy-of-the-command-s-seeded-repository-facts ^t-41f50d
  - native_change: prefetch-workers-share-one-copy-of-the-command-s-seeded-repository-facts
  - revision: 2915d230bb7fdfd75aebe1c9e04f29156d6470e4
  - holds: accept
  - drives: review
  - by: claude-code (hc t38p)
