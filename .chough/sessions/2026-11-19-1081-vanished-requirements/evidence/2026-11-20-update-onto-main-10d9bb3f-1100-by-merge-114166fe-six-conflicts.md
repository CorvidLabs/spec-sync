---
kind: evidence
id: 2026-11-20-update-onto-main-10d9bb3f-1100-by-merge-114166fe-six-conflicts
aliases: []
claim: "Update onto main 10d9bb3f (#1100) by merge 114166fe. Six conflicts, each two additions at one place, resolved by keeping both sides whole in id order: hi/change.md (CHANGE-15, then CHANGE-17), change.3md (z=55, then z=56), REQ-change-163 then 169, REQ-cmd-change-040 then 042, tests/integration.rs, and INTENT.md's count (59). ss-lead's check_review_carry.py found the branch's 30 reviewed files word for word the same; the INTENT count line is the only difference. change check --commit re-recorded verification (706be086, 24cd747c). Real repos (rule 3) on the merged build: starling PR #43's last open commit 3b1dca0 is refused (exit 1, 0.4 s), naming all 26 lost REQ-StarlingCore ids in natural order (6.3.0 passes). change audit --strict gives the same exit codes and errors as 6.3.0 on spec-sync main 10d9bb3f, Chough main 4c764c09, Chough #311 (75b6bf6e) and #310 (af794540), starling c2863c8 and two other projects with open changes. fledge lanes run verify passed at 24cd747c through one-lane (3114 unit, 645 integration, 62/62 specs, 100% coverage); change audit --strict exit 0; pre-push and fledge trust verify passed."
result: passed
evidence_kind: product-test
revision: 24cd747c2038b6aa53c6bf28455d15210d77ef56
content: a8d869c57b748f9757425f4b7257b28ae72ef3f130ddc8800cf49a2c72f600dc
environment: local macOS 26, rust 1.89.0, branch 1081-vanished-requirements at 24cd747c
criteria: [CHANGE-17, CHANGE-17.a, CHANGE-17.b, CHANGE-17.c]
requirements: [REQ-change-169]
contracts: []
limitations: []
observed: 2026-11-20T01:07:11Z
actor:
  - agent: claude-code
    terminal: t4t
---
**Update onto main 10d9bb3f (#1100) by merge 114166fe. Six conflicts, each two additions at one place, resolved by keeping both sides whole in id order: hi/change.md (CHANGE-15, then CHANGE-17), change.3md (z=55, then z=56), REQ-change-163 then 169, REQ-cmd-change-040 then 042, tests/integration.rs, and INTENT.md's count (59). ss-lead's check_review_carry.py found the branch's 30 reviewed files word for word the same; the INTENT count line is the only difference. change check --commit re-recorded verification (706be086, 24cd747c). Real repos (rule 3) on the merged build: starling PR #43's last open commit 3b1dca0 is refused (exit 1, 0.4 s), naming all 26 lost REQ-StarlingCore ids in natural order (6.3.0 passes). change audit --strict gives the same exit codes and errors as 6.3.0 on spec-sync main 10d9bb3f, Chough main 4c764c09, Chough #311 (75b6bf6e) and #310 (af794540), starling c2863c8 and two other projects with open changes. fledge lanes run verify passed at 24cd747c through one-lane (3114 unit, 645 integration, 62/62 specs, 100% coverage); change audit --strict exit 0; pre-push and fledge trust verify passed.**

Recorded as passed in local macOS 26, rust 1.89.0, branch 1081-vanished-requirements at 24cd747c at revision 24cd747c2038, observed 2026-11-20T01:07:11Z.
