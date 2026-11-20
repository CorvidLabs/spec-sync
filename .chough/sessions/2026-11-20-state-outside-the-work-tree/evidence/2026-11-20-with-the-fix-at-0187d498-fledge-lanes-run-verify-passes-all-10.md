---
kind: evidence
id: 2026-11-20-with-the-fix-at-0187d498-fledge-lanes-run-verify-passes-all-10
aliases: []
claim: "With the fix at 0187d498, fledge lanes run verify passes all 10 steps: unit 3122 passed, 0 failed, 2 ignored; integration 697 passed, 0 failed; clippy on all targets, the release build, the strict spec check at 62 specs and 100% coverage, and the gate scripts. All 5 new unit tests and 13 new integration tests pass (also REQ-hash-cache-009, REQ-cmd-check-021, REQ-rehash-003, REQ-cmd-resolve-002, REQ-cmd-resolve-003), the scans find nothing unmarked, and the older tests that assumed .specsync/hashes.json and the in-tree journal now point at the local state folder."
result: passed
evidence_kind: product-test
revision: 0187d49824c69d81cffd51d68b24bfd28afc6765
content: 2345cacabed0e5d381e05f4a2424e20aee042726356f24191ed679d284189a66
environment: macOS arm64 (APFS), rustc 1.89.0, isolated git config, through one-lane; tip 0187d498 (change 6's code f6eac853, 1a5adcdd, 93e06a8e, with change 5 re-verified on it)
fingerprint: sha256:9773294cafecdc81155c0bd80b7e3b8640bf9ae5854af3b566af31254ec470e5
external: ext-ceafc1
criteria: [SETUP-6, SETUP-7, CHECK-1.f]
requirements:
  - REQ-util-006
  - REQ-util-007
  - REQ-util-008
  - REQ-change-162
  - REQ-hooks-009
  - REQ-validator-052
contracts: []
limitations: []
observed: 2026-11-20T02:51:38Z
actor:
  - agent: claude-code
    terminal: t5r
---
**With the fix at 0187d498, fledge lanes run verify passes all 10 steps: unit 3122 passed, 0 failed, 2 ignored; integration 697 passed, 0 failed; clippy on all targets, the release build, the strict spec check at 62 specs and 100% coverage, and the gate scripts. All 5 new unit tests and 13 new integration tests pass (also REQ-hash-cache-009, REQ-cmd-check-021, REQ-rehash-003, REQ-cmd-resolve-002, REQ-cmd-resolve-003), the scans find nothing unmarked, and the older tests that assumed .specsync/hashes.json and the in-tree journal now point at the local state folder.**

Recorded as passed in macOS arm64 (APFS), rustc 1.89.0, isolated git config, through one-lane; tip 0187d498 (change 6's code f6eac853, 1a5adcdd, 93e06a8e, with change 5 re-verified on it) at revision 0187d49824c6, observed 2026-11-20T02:51:38Z.
