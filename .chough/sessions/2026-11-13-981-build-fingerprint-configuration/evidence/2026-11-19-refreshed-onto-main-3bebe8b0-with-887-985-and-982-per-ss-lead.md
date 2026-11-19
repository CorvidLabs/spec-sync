---
kind: evidence
id: 2026-11-19-refreshed-onto-main-3bebe8b0-with-887-985-and-982-per-ss-lead
aliases: []
claim: "Refreshed onto main 3bebe8b0 with #887, #985 and #982. Per ss-lead's option (1): the validation snapshot's input digest frames cfg!(debug_assertions) at run time; the configuration test asserts the four names build.rs frames; the read cache's identity half is issue #1098 (src/change/read_cache.rs is outside #981's declared paths). the_build_fingerprint_frames_the_compile_configuration, a_snapshot_from_a_build_with_other_debug_assertions_is_not_replayed and both build_fingerprint tests pass. A cache written under the old fingerprint is ignored and replaced, not deleted. fledge lanes run verify at 42dda911 with GIT_CONFIG_GLOBAL=/dev/null (#1092): 10/10 steps, 3085 unit and 627 integration tests, strict specs 62/62, 100% file and LOC coverage; lifecycle_commands_stay_within_git_spawn_ceilings passes; change audit --strict and pre-push passed."
result: passed
evidence_kind: product-test
revision: 42dda911e039dd4b032fff5a278f1fc2eee391cb
content: 37d3903c9d3468a39a7fcfcac1aac4721a47c2b57679a3de8bad06b2e6b4f189
environment: local macOS 26, rust 1.89.0, branch fix/read-cache-887-985-981-982 at 42dda911
criteria: [SPEED-1.c]
requirements: []
contracts: []
limitations: []
observed: 2026-11-19T08:44:27Z
actor:
  - agent: claude-code
    terminal: t4t
---
**Refreshed onto main 3bebe8b0 with #887, #985 and #982. Per ss-lead's option (1): the validation snapshot's input digest frames cfg!(debug_assertions) at run time; the configuration test asserts the four names build.rs frames; the read cache's identity half is issue #1098 (src/change/read_cache.rs is outside #981's declared paths). the_build_fingerprint_frames_the_compile_configuration, a_snapshot_from_a_build_with_other_debug_assertions_is_not_replayed and both build_fingerprint tests pass. A cache written under the old fingerprint is ignored and replaced, not deleted. fledge lanes run verify at 42dda911 with GIT_CONFIG_GLOBAL=/dev/null (#1092): 10/10 steps, 3085 unit and 627 integration tests, strict specs 62/62, 100% file and LOC coverage; lifecycle_commands_stay_within_git_spawn_ceilings passes; change audit --strict and pre-push passed.**

Recorded as passed in local macOS 26, rust 1.89.0, branch fix/read-cache-887-985-981-982 at 42dda911 at revision 42dda911e039, observed 2026-11-19T08:44:27Z.
