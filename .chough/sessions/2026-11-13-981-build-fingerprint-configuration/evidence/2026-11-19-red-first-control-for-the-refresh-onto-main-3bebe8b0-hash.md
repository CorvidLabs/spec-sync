---
kind: evidence
id: 2026-11-19-red-first-control-for-the-refresh-onto-main-3bebe8b0-hash
aliases: []
claim: "Red-first control for the refresh onto main 3bebe8b0: hash_cache::tests::the_build_fingerprint_frames_the_compile_configuration (the option_env! version from d6489e8f) fails, the defect #981 reports. Also measured: cargo 1.89.0 passes a build script PROFILE and DEBUG but no CARGO_CFG_DEBUG_ASSERTIONS (three-line probe crate), so the branch's final test, which asserted it, failed at the batch tip 5587c03b; a_snapshot_from_a_build_with_other_debug_assertions_is_not_replayed failed before the snapshot input digest framed cfg!(debug_assertions)."
result: failed
evidence_kind: product-test
revision: 42dda911e039dd4b032fff5a278f1fc2eee391cb
content: 37d3903c9d3468a39a7fcfcac1aac4721a47c2b57679a3de8bad06b2e6b4f189
environment: "red control: main 3bebe8b0 plus only the src parts of the 11-13 failing-test commits (63e8a7c2, d0c71f54, d6489e8f, 07c15e4b)"
criteria: [SPEED-1.c]
requirements: []
contracts: []
limitations: []
observed: 2026-11-19T08:44:27Z
actor:
  - agent: claude-code
    terminal: t4t
---
**Red-first control for the refresh onto main 3bebe8b0: hash_cache::tests::the_build_fingerprint_frames_the_compile_configuration (the option_env! version from d6489e8f) fails, the defect #981 reports. Also measured: cargo 1.89.0 passes a build script PROFILE and DEBUG but no CARGO_CFG_DEBUG_ASSERTIONS (three-line probe crate), so the branch's final test, which asserted it, failed at the batch tip 5587c03b; a_snapshot_from_a_build_with_other_debug_assertions_is_not_replayed failed before the snapshot input digest framed cfg!(debug_assertions).**

Recorded as failed in red control: main 3bebe8b0 plus only the src parts of the 11-13 failing-test commits (63e8a7c2, d0c71f54, d6489e8f, 07c15e4b) at revision 42dda911e039, observed 2026-11-19T08:44:27Z.

## Key failing lines

- the_build_fingerprint_frames_the_compile_configuration FAILED; at 5587c03b: left false, right true (CARGO_CFG_DEBUG_ASSERTIONS)
