---
kind: evidence
id: 2026-11-07-fledge-lanes-run-verify-was-interrupted-by-the-orc-s-safe-park
aliases: [OBS-product-test-f6bd8d6b1f50fef57acf10bb]
claim: fledge lanes run verify was interrupted by the orc's SAFE-PARK instruction and exited 130. The lane did not complete, so there is no full-verify verdict.
result: not-run
evidence_kind: product-test
method: automated
revision: 0391a4c8d021d6ebb53086a849ad23395faefca8
content: 3c5a49fb5f53e860a25f80e49773dfa0fbe80f720436b82cded6a7d02761b4b2
environment: local-macos-arm64
criteria: [MERGE-1]
requirements: [REQ-cmd-merge-003]
contracts: [specs/cmd_merge/cmd_merge.spec.md]
limitations:
  - Interrupted receipt only; rerun fledge lanes run verify on resume. Focused install tests and the original command scenario passed separately.
observed: 2026-11-07T15:36:13Z
---
**fledge lanes run verify was interrupted by the orc's SAFE-PARK instruction and exited 130. The lane did not complete, so there is no full-verify verdict.**

Recorded as not-run in local-macos-arm64 at revision 0391a4c8d021, observed 2026-11-07T15:36:13Z.
