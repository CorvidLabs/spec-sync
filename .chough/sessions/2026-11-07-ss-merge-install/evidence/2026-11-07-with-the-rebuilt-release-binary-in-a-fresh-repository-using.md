---
kind: evidence
id: 2026-11-07-with-the-rebuilt-release-binary-in-a-fresh-repository-using
aliases: [OBS-product-test-e62f4a4b773df570bf80de09]
claim: "With the rebuilt release binary in a fresh repository using canonical .specsync/config.toml specs_dir=., merge --install emitted **/*.spec.md and **/requirements.md, and git check-attr returned merge: specsync for root.spec.md, nested/root.spec.md, requirements.md, and nested/requirements.md."
result: passed
evidence_kind: product-test
method: manual
revision: 0391a4c8d021d6ebb53086a849ad23395faefca8
content: dd54d685dd40b9e5fca7fce6d9defde55c009637d1acc0f4f7254113f709b078
environment: local-macos-arm64
criteria: [MERGE-1]
requirements: [REQ-cmd-merge-003]
contracts: [specs/cmd_merge/cmd_merge.spec.md]
limitations:
  - Linked to existing REQ-cmd-merge-003 until approved REQ-cmd-merge-004 is materialized by change check.
observed: 2026-11-07T15:15:38Z
---
**With the rebuilt release binary in a fresh repository using canonical .specsync/config.toml specs_dir=., merge --install emitted **/*.spec.md and **/requirements.md, and git check-attr returned merge: specsync for root.spec.md, nested/root.spec.md, requirements.md, and nested/requirements.md.**

Recorded as passed in local-macos-arm64 at revision 0391a4c8d021, observed 2026-11-07T15:15:38Z.
