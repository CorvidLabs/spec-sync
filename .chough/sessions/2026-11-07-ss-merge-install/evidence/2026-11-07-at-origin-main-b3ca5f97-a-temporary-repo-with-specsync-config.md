---
kind: evidence
id: 2026-11-07-at-origin-main-b3ca5f97-a-temporary-repo-with-specsync-config
aliases: [OBS-product-test-b47ebf5a0d626500e6c08d9c]
claim: "At origin/main b3ca5f97, a temporary repo with .specsync/config.toml specs_dir=. ran merge --install successfully but wrote ./**/*.spec.md and ./**/requirements.md; git check-attr reported merge: unspecified for root and nested .spec.md and requirements.md paths."
result: failed
evidence_kind: product-test
method: manual
revision: b3ca5f97aa9c07b09ab6bfd8d7eece28f2c30f79
content: 1ebb3fcf2479c84274db30d5520dbd0018d62d483218fdeeb41a3cf123523605
environment: local
criteria: [MERGE-1]
requirements: [REQ-cmd-merge-003]
contracts: [specs/cmd_merge/cmd_merge.spec.md]
limitations: [Manual reproduction only; the red automated regression test is not yet written.]
observed: 2026-11-07T03:10:59Z
---
**At origin/main b3ca5f97, a temporary repo with .specsync/config.toml specs_dir=. ran merge --install successfully but wrote ./**/*.spec.md and ./**/requirements.md; git check-attr reported merge: unspecified for root and nested .spec.md and requirements.md paths.**

Recorded as failed in local at revision b3ca5f97aa9c, observed 2026-11-07T03:10:59Z.
