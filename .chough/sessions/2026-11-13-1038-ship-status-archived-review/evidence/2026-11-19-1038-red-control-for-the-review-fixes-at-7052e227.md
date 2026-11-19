---
kind: evidence
id: 2026-11-19-1038-red-control-for-the-review-fixes-at-7052e227
aliases: []
claim: "Red-first control for the review of PR #1093, at 7052e227 with only the new tests: an_archive_tip_whose_package_is_still_present_is_product fails (a tip that modifies the change's package, still present, reads archive_only where CI picks the product lane), and a_workflow_v1_archive_is_not_done_by_finalize_and_suggests_no_review fails (a workflow-v1 archive holding a review.json reads review_currency current). These are the two review findings, fixed in c697bbde; not new bugs."
result: failed
evidence_kind: product-test
revision: 612d255dc6c463fdabdb7fe329f74dcb78de5824
content: d6b777e99a4073e1fc42bb05499eea290730b7e3889e0a0d0ac54d84e020b7d6
environment: macOS 27.2 (Darwin 27.2.0), Apple M5 Ultra, local clone
fingerprint: sha256:11db5a78ddb4a7d31ec7cb8bd15a1aa36977c5bb44853780ede1882dc2bffebc
external: ext-985861
criteria: [SHIP-8]
requirements: []
contracts: [specs/cmd_change/cmd_change.spec.md, specs/change/change.spec.md]
limitations: []
observed: 2026-11-19T05:56:26Z
actor:
  - agent: claude-code
    terminal: t59
---
**Red-first control for the review of PR #1093, at 7052e227 with only the new tests: an_archive_tip_whose_package_is_still_present_is_product fails (a tip that modifies the change's package, still present, reads archive_only where CI picks the product lane), and a_workflow_v1_archive_is_not_done_by_finalize_and_suggests_no_review fails (a workflow-v1 archive holding a review.json reads review_currency current). These are the two review findings, fixed in c697bbde; not new bugs.**

Recorded as failed in macOS 27.2 (Darwin 27.2.0), Apple M5 Ultra, local clone at revision 612d255dc6c4, observed 2026-11-19T05:56:26Z.
