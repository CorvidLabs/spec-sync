---
kind: evidence
id: 2026-11-07-after-the-root-normalization-fix-cargo-test-bin-specsync
aliases: [OBS-product-test-8b3cf28186053c41615bf658]
claim: After the root normalization fix, cargo test --bin specsync install_tests passed 6/6 tests, including root_specs_dir_matches_contracts for both . and ./ across root and nested spec and requirements paths; 2893 tests were filtered out.
result: passed
evidence_kind: product-test
method: automated
revision: 0391a4c8d021d6ebb53086a849ad23395faefca8
content: dd54d685dd40b9e5fca7fce6d9defde55c009637d1acc0f4f7254113f709b078
environment: local-macos-arm64
criteria: [MERGE-1]
requirements: [REQ-cmd-merge-003]
contracts: [specs/cmd_merge/cmd_merge.spec.md]
limitations:
  - Linked to existing REQ-cmd-merge-003 until approved REQ-cmd-merge-004 is materialized by change check.
observed: 2026-11-07T15:15:35Z
---
**After the root normalization fix, cargo test --bin specsync install_tests passed 6/6 tests, including root_specs_dir_matches_contracts for both . and ./ across root and nested spec and requirements paths; 2893 tests were filtered out.**

Recorded as passed in local-macos-arm64 at revision 0391a4c8d021, observed 2026-11-07T15:15:35Z.
