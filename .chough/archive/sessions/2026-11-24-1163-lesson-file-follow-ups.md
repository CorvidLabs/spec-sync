---
kind: session
id: 2026-11-24-1163-lesson-file-follow-ups
aliases: [1163-lesson-file-follow-ups]
objective: "check reports a broken lesson file on a cached run, under the module's own name, and when it is not a regular file (review findings on #1163)"
modules: [cmd_check, hash_cache]
stage: done
stage_status: archived
iteration: 1
native_change: check-reports-a-broken-lesson-file-on-a-cached-run-under-the-module-s-own-name-and-when-it-is-not-a-regular-file
criteria: [THREE-1]
observations:
  - 2026-11-24-red-control-at-24d553f6-the-published-head-of-1163-with-only
  - 2026-11-24-at-4d7ffd5a-the-fixes-on-1163-s-head-fledge-lanes-run-verify
summary:
  - "Three review findings on #1163's lesson-file warning, each reproduced on the CLI: a cached check missed it, the file was named from the spec file rather than the module, and a directory of that name was skipped."
closed: 2026-11-25
outcome: done
---
# check reports a broken lesson file on a cached run, under the module's own name, and when it is not a regular file (review findings on #1163)

## Goal

check reports a broken lesson file on a cached run, under the module's own name, and when it is not a regular file (review findings on #1163)

## Where it stands

## Decisions and why

- 0xLeif decided "Scope approval of check-reports-a-broken-lesson-file-on-a-cached-run-under-the-module-s-own-name-and-when-it-is-not-a-regular-file": "I approve all three" (Claude Code release-lead chat, 2026-11-24 UTC, typed by him to the agent recording it. It answered a message that asked three numbered questions. The third was '#1163 successor: do you approve these requirement words?' and quoted them: REQ-cmd-check-023 amended (the file is the one named for the spec's module, an entry of that name that is not a regular file is reported as unreadable, and a strict lifecycle check fails on the warning as on any other, with three added acceptance criteria), REQ-hash-cache-011 added in full, and the validator Error Cases row. He had already chosen, earlier in the same chat, to fix the three findings in one successor change and keep change check --strict failing.); recorded by claude-code-t14v; a relayed claim, not authenticated; ran specsync change approve check-reports-a-broken-lesson-file-on-a-cached-run-under-the-module-s-own-name-and-when-it-is-not-a-regular-file as 0xLeif
- 0xLeif decided "Implementation review of check-reports-a-broken-lesson-file-on-a-cached-run-under-the-module-s-own-name-and-when-it-is-not-a-regular-file": "I accept" (Claude Code release-lead chat, 2026-11-25 UTC, typed by him to the agent recording it. It answered a message that asked one question: whether he accepts the implementation of the #1163 follow-up change. He had been shown the native Verify result, the four tests that failed on #1163's published head and pass now, the four review findings re-run on the CLI, the size of the change and the one hardening beyond the findings. That evidence was from the branch before main's #1166 was merged into it. The merge changed no source, test or Cargo path, and the full native Verify lane passed again on the merged head 933f6f0b before this record was made.); recorded by claude-code-t14v; a relayed claim, not authenticated; the scoped review of check-reports-a-broken-lesson-file-on-a-cached-run-under-the-module-s-own-name-and-when-it-is-not-a-regular-file was already current, so no review ran

## What was verified

- Red control at 24d553f6 (the published head of #1163) with only the new tests added: a_cached_check_reports_a_3md_broken_since_the_last_run failed with 'Skipped 1 unchanged spec(s)', the_3md_checked_is_named_for_the_module_not_the_spec_file failed because main.3md was read, a_directory_named_as_the_3md_is_reported failed with nothing reported, and the unit test a_snapshot_is_not_replayed_over_a_changed_lesson_file failed at 'an added lesson file re-validates'. The two earlier lesson-file tests passed. - dc29995281f0 ^2026-11-24-red-control-at-24d553f6-the-published-head-of-1163-with-only
- At 4d7ffd5a (the fixes on #1163's head), fledge lanes run verify passed 10/10 steps in 30m54s: fmt, lint and check-types, cargo test (unit 3358 passed, 2 ignored; integration 829 passed), the release build, the forced strict spec check and the six CI contract suites. No src, tests or Cargo path changed between that commit and aeabda2a, where the deltas are materialized and the strict check, change audit, doctor, pre-push and Trust (Augur review, risk 37) pass. On the CLI with the release binary that lane built (sha256 d4bbbec3), against the binary of #1163's head: a plain check after a clean run reports a lesson file broken since (it was skipped before); a directory named <module>.3md is reported as not a regular file (nothing was said before); for main.spec.md with module product, product.3md is the file checked and main.3md is not read (the reverse before); a FIFO or dangling link there still fails closed in discovery; change check passes and change check --strict fails on the warning, on both. - 44000d8701e4 ^2026-11-24-at-4d7ffd5a-the-fixes-on-1163-s-head-fledge-lanes-run-verify

## Bugs

None.

## Open threads

## Timeline

- 2026-11-24T22:29:19Z · start · intent → intent · check reports a broken lesson file on a cached run, under the module's own name, and when it is not a regular file (review findings on #1163) ^t-5e4a8a
  - revision: 24d553f6912b7594e8664e629eba5092afd6cbe2
  - by: claude-code (hc t14v)
- 2026-11-24T22:53:01Z · advance · intent → design ·  ^t-46b427
  - native_change: check-reports-a-broken-lesson-file-on-a-cached-run-under-the-module-s-own-name-and-when-it-is-not-a-regular-file
  - revision: 24d553f6912b7594e8664e629eba5092afd6cbe2
  - by: claude-code (hc t14v)
- 2026-11-24T22:53:01Z · skip · design → plan · Fixes to an existing warning and its cache input. No visuals or interactions. ^t-91def8
  - native_change: check-reports-a-broken-lesson-file-on-a-cached-run-under-the-module-s-own-name-and-when-it-is-not-a-regular-file
  - revision: 24d553f6912b7594e8664e629eba5092afd6cbe2
  - by: claude-code (hc t14v)
- 2026-11-24T22:53:01Z · gate-opened · plan → plan · Scope approval of check-reports-a-broken-lesson-file-on-a-cached-run-under-the-module-s-own-name-and-when-it-is-not-a-regular-file ^t-5defe1
  - native_change: check-reports-a-broken-lesson-file-on-a-cached-run-under-the-module-s-own-name-and-when-it-is-not-a-regular-file
  - revision: 24d553f6912b7594e8664e629eba5092afd6cbe2
  - holds: plan
  - drives: approve
  - by: claude-code (hc t14v)
- 2026-11-24T22:53:04Z · direct-native-write · plan → plan · check-reports-a-broken-lesson-file-on-a-cached-run-under-the-module-s-own-name-and-when-it-is-not-a-regular-file's lifecycle records changed outside Chough: state.json changed, approvals.json changed; state draft → approved ^t-e2a9eb
  - native_change: check-reports-a-broken-lesson-file-on-a-cached-run-under-the-module-s-own-name-and-when-it-is-not-a-regular-file
  - revision: 24d553f6912b7594e8664e629eba5092afd6cbe2
  - direct_write:
    - change: check-reports-a-broken-lesson-file-on-a-cached-run-under-the-module-s-own-name-and-when-it-is-not-a-regular-file
    - files:
      - state.json: changed
      - approvals.json: changed
    - before_state: draft
    - after_state: approved
  - by: claude-code (hc t14v)
- 2026-11-24T22:53:04Z · gate-closed · plan → plan · Scope approval of check-reports-a-broken-lesson-file-on-a-cached-run-under-the-module-s-own-name-and-when-it-is-not-a-regular-file ^t-475c50
  - native_change: check-reports-a-broken-lesson-file-on-a-cached-run-under-the-module-s-own-name-and-when-it-is-not-a-regular-file
  - revision: 24d553f6912b7594e8664e629eba5092afd6cbe2
  - relayed_by:
    - decided_by: 0xLeif
    - words: I approve all three
    - where: Claude Code release-lead chat, 2026-11-24 UTC, typed by him to the agent recording it. It answered a message that asked three numbered questions. The third was '#1163 successor: do you approve these requirement words?' and quoted them: REQ-cmd-check-023 amended (the file is the one named for the spec's module, an entry of that name that is not a regular file is reported as unreadable, and a strict lifecycle check fails on the warning as on any other, with three added acceptance criteria), REQ-hash-cache-011 added in full, and the validator Error Cases row. He had already chosen, earlier in the same chat, to fix the three findings in one successor change and keep change check --strict failing.
    - recorded_by: claude-code-t14v
    - native: specsync change approve check-reports-a-broken-lesson-file-on-a-cached-run-under-the-module-s-own-name-and-when-it-is-not-a-regular-file
    - actor: 0xLeif
  - by: claude-code (hc t14v)
- 2026-11-24T22:53:08Z · advance · plan → build · native:approved ^t-5cb4ab
  - native_change: check-reports-a-broken-lesson-file-on-a-cached-run-under-the-module-s-own-name-and-when-it-is-not-a-regular-file
  - gate: native:approved
  - revision: 24d553f6912b7594e8664e629eba5092afd6cbe2
  - by: claude-code (hc t14v)
- 2026-11-24T23:27:01Z · advance · build → test · native:verifying ^t-639f1c
  - native_change: check-reports-a-broken-lesson-file-on-a-cached-run-under-the-module-s-own-name-and-when-it-is-not-a-regular-file
  - gate: native:verifying
  - revision: aeabda2a3983818df26bdf732c1aef85f3c64aed
  - by: claude-code (hc t14v)
- 2026-11-24T23:27:03Z · advance · test → accept · native:verifying ^t-c1de99
  - native_change: check-reports-a-broken-lesson-file-on-a-cached-run-under-the-module-s-own-name-and-when-it-is-not-a-regular-file
  - gate: native:verifying
  - revision: aeabda2a3983818df26bdf732c1aef85f3c64aed
  - by: claude-code (hc t14v)
- 2026-11-24T23:27:03Z · gate-opened · accept → accept · Implementation review of check-reports-a-broken-lesson-file-on-a-cached-run-under-the-module-s-own-name-and-when-it-is-not-a-regular-file ^t-ca692f
  - native_change: check-reports-a-broken-lesson-file-on-a-cached-run-under-the-module-s-own-name-and-when-it-is-not-a-regular-file
  - revision: aeabda2a3983818df26bdf732c1aef85f3c64aed
  - holds: accept
  - drives: review
  - by: claude-code (hc t14v)
- 2026-11-25T02:23:52Z · direct-native-write · accept → accept · check-reports-a-broken-lesson-file-on-a-cached-run-under-the-module-s-own-name-and-when-it-is-not-a-regular-file's lifecycle records changed outside Chough: review.json added, review-attempts.json added; state stays verifying ^t-8f5d54
  - native_change: check-reports-a-broken-lesson-file-on-a-cached-run-under-the-module-s-own-name-and-when-it-is-not-a-regular-file
  - revision: 22b1139f6a08101fc90c9eb456a0125955c85a3e
  - direct_write:
    - change: check-reports-a-broken-lesson-file-on-a-cached-run-under-the-module-s-own-name-and-when-it-is-not-a-regular-file
    - files:
      - review.json: added
      - review-attempts.json: added
    - before_state: verifying
    - after_state: verifying
  - by: claude-code (hc t14v)
- 2026-11-25T02:23:52Z · gate-closed · accept → accept · Implementation review of check-reports-a-broken-lesson-file-on-a-cached-run-under-the-module-s-own-name-and-when-it-is-not-a-regular-file ^t-fbfd7b
  - native_change: check-reports-a-broken-lesson-file-on-a-cached-run-under-the-module-s-own-name-and-when-it-is-not-a-regular-file
  - revision: 22b1139f6a08101fc90c9eb456a0125955c85a3e
  - relayed_by:
    - decided_by: 0xLeif
    - words: I accept
    - where: Claude Code release-lead chat, 2026-11-25 UTC, typed by him to the agent recording it. It answered a message that asked one question: whether he accepts the implementation of the #1163 follow-up change. He had been shown the native Verify result, the four tests that failed on #1163's published head and pass now, the four review findings re-run on the CLI, the size of the change and the one hardening beyond the findings. That evidence was from the branch before main's #1166 was merged into it. The merge changed no source, test or Cargo path, and the full native Verify lane passed again on the merged head 933f6f0b before this record was made.
    - recorded_by: claude-code-t14v
  - by: claude-code (hc t14v)
- 2026-11-25T02:23:54Z · advance · accept → done · native:archived ^t-d051c3
  - native_change: check-reports-a-broken-lesson-file-on-a-cached-run-under-the-module-s-own-name-and-when-it-is-not-a-regular-file
  - gate: native:archived
  - revision: 22b1139f6a08101fc90c9eb456a0125955c85a3e
  - by: claude-code (hc t14v)

Closed 2026-11-25: done.

## Receipts

```yaml
receipts:
  - id: 2026-11-24-at-4d7ffd5a-the-fixes-on-1163-s-head-fledge-lanes-run-verify
    claim: "At 4d7ffd5a (the fixes on #1163's head), fledge lanes run verify passed 10/10 steps in 30m54s: fmt, lint and check-types, cargo test (unit 3358 passed, 2 ignored; integration 829 passed), the release build, the forced strict spec check and the six CI contract suites. No src, tests or Cargo path changed between that commit and aeabda2a, where the deltas are materialized and the strict check, change audit, doctor, pre-push and Trust (Augur review, risk 37) pass. On the CLI with the release binary that lane built (sha256 d4bbbec3), against the binary of #1163's head: a plain check after a clean run reports a lesson file broken since (it was skipped before); a directory named <module>.3md is reported as not a regular file (nothing was said before); for main.spec.md with module product, product.3md is the file checked and main.3md is not read (the reverse before); a FIFO or dangling link there still fails closed in discovery; change check passes and change check --strict fails on the warning, on both."
    result: passed
    revision: aeabda2a3983818df26bdf732c1aef85f3c64aed
    environment: macOS 27.2 (Darwin 27.2.0), Apple M5 Ultra, local clone, one Cargo job and two test threads
    content: 249d7216b1e401b6abb655cc28397d6d48ef36f70a96b082f233796f1b4c4255
    fingerprint: sha256:44000d8701e4c3ab7c4f5f054e5abc63e2a888d072a56f6765f4fc68c7eaa9a9
    criteria: THREE-1
    observed: 2026-11-24T23:27:01Z
    actor: claude-code (hc t14v)
  - id: 2026-11-24-red-control-at-24d553f6-the-published-head-of-1163-with-only
    claim: "Red control at 24d553f6 (the published head of #1163) with only the new tests added: a_cached_check_reports_a_3md_broken_since_the_last_run failed with 'Skipped 1 unchanged spec(s)', the_3md_checked_is_named_for_the_module_not_the_spec_file failed because main.3md was read, a_directory_named_as_the_3md_is_reported failed with nothing reported, and the unit test a_snapshot_is_not_replayed_over_a_changed_lesson_file failed at 'an added lesson file re-validates'. The two earlier lesson-file tests passed."
    result: failed
    revision: aeabda2a3983818df26bdf732c1aef85f3c64aed
    environment: macOS 27.2 (Darwin 27.2.0), Apple M5 Ultra, local clone, one Cargo job and two test threads
    content: 249d7216b1e401b6abb655cc28397d6d48ef36f70a96b082f233796f1b4c4255
    fingerprint: sha256:dc29995281f0f9d0c4eb7143c2ca3742fa14c3ea4ed803256fbdecb9f850667d
    criteria: THREE-1
    observed: 2026-11-24T23:27:01Z
    actor: claude-code (hc t14v)
```
