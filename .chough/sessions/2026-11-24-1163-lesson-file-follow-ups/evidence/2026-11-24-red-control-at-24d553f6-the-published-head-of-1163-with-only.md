---
kind: evidence
id: 2026-11-24-red-control-at-24d553f6-the-published-head-of-1163-with-only
aliases: []
claim: "Red control at 24d553f6 (the published head of #1163) with only the new tests added: a_cached_check_reports_a_3md_broken_since_the_last_run failed with 'Skipped 1 unchanged spec(s)', the_3md_checked_is_named_for_the_module_not_the_spec_file failed because main.3md was read, a_directory_named_as_the_3md_is_reported failed with nothing reported, and the unit test a_snapshot_is_not_replayed_over_a_changed_lesson_file failed at 'an added lesson file re-validates'. The two earlier lesson-file tests passed."
result: failed
evidence_kind: product-test
revision: aeabda2a3983818df26bdf732c1aef85f3c64aed
content: 249d7216b1e401b6abb655cc28397d6d48ef36f70a96b082f233796f1b4c4255
environment: macOS 27.2 (Darwin 27.2.0), Apple M5 Ultra, local clone, one Cargo job and two test threads
fingerprint: sha256:dc29995281f0f9d0c4eb7143c2ca3742fa14c3ea4ed803256fbdecb9f850667d
external: ext-96f985
criteria: [THREE-1]
requirements: []
contracts: []
limitations: []
observed: 2026-11-24T23:27:01Z
actor:
  - agent: claude-code
    terminal: t14v
---
**Red control at 24d553f6 (the published head of #1163) with only the new tests added: a_cached_check_reports_a_3md_broken_since_the_last_run failed with 'Skipped 1 unchanged spec(s)', the_3md_checked_is_named_for_the_module_not_the_spec_file failed because main.3md was read, a_directory_named_as_the_3md_is_reported failed with nothing reported, and the unit test a_snapshot_is_not_replayed_over_a_changed_lesson_file failed at 'an added lesson file re-validates'. The two earlier lesson-file tests passed.**

Recorded as failed in macOS 27.2 (Darwin 27.2.0), Apple M5 Ultra, local clone, one Cargo job and two test threads at revision aeabda2a3983, observed 2026-11-24T23:27:01Z.

## Key failing lines

- a lesson file added broken after a clean run must be reported: Skipped 1 unchanged spec(s)
