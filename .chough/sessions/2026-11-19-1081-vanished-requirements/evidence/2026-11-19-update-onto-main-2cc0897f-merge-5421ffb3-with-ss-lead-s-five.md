---
kind: evidence
id: 2026-11-19-update-onto-main-2cc0897f-merge-5421ffb3-with-ss-lead-s-five
aliases: []
claim: "Update onto main 2cc0897f (merge 5421ffb3) with ss-lead's five review points and the tester's run (t4p passed b8018cc0). Red first: 4347e4aa's tests failed five, each for its point; the depth-1 clone with no origin ref already skipped with the note and is now pinned. Fix b2600d14: the walk and the changed-path list skip .git, .specsync and exclude_dirs (specs_dir '.' plus a change workspace and a vendored copy no longer mask a loss, and the artifact's own id is not read at the reference); an unreadable change accounts for no removal and the lost ids are still named; ids sort naturally; every Git read failure names repair or 'git fetch origin', and 'git fetch --unshallow origin' in a shallow clone missing the merge-base; ship-status exits 1 while the blocker holds (asserted directly, #654 on main). Real repos: starling PR #43's last open commit refused in 1.05 s naming all 26 ids in natural order (6.3.0 passes); Chough #311 and #310 and the mains of Chough 4c764c09, starling db97480 and spec-sync ee76ef1b give the same results as 6.3.0. Timing on spec-sync, the largest: change audit 4.9 s against 6.3.0's 5.0 s; a root-specs walk there would cover 1,901 directories outside .git and .specsync (find: 0.15 s). fledge lanes run verify passed at 967e6bdb (3107 unit, 630 integration, 62/62 specs, 100% coverage); change audit --strict exit 0; pre-push and fledge trust verify passed."
result: passed
evidence_kind: product-test
revision: 967e6bdb34dd1d152d50a42b88f9a2390ed1e114
content: 044d76bbcc4d594f73aa28741f819d70a8676a00bd9b1f8ed4c751af663b76a3
environment: local macOS 26, rust 1.89.0, branch 1081-vanished-requirements at 967e6bdb
criteria: [CHANGE-17, CHANGE-17.a, CHANGE-17.b, CHANGE-17.c]
requirements: [REQ-change-169]
contracts: []
limitations: []
observed: 2026-11-19T16:16:02Z
actor:
  - agent: claude-code
    terminal: t4t
---
**Update onto main 2cc0897f (merge 5421ffb3) with ss-lead's five review points and the tester's run (t4p passed b8018cc0). Red first: 4347e4aa's tests failed five, each for its point; the depth-1 clone with no origin ref already skipped with the note and is now pinned. Fix b2600d14: the walk and the changed-path list skip .git, .specsync and exclude_dirs (specs_dir '.' plus a change workspace and a vendored copy no longer mask a loss, and the artifact's own id is not read at the reference); an unreadable change accounts for no removal and the lost ids are still named; ids sort naturally; every Git read failure names repair or 'git fetch origin', and 'git fetch --unshallow origin' in a shallow clone missing the merge-base; ship-status exits 1 while the blocker holds (asserted directly, #654 on main). Real repos: starling PR #43's last open commit refused in 1.05 s naming all 26 ids in natural order (6.3.0 passes); Chough #311 and #310 and the mains of Chough 4c764c09, starling db97480 and spec-sync ee76ef1b give the same results as 6.3.0. Timing on spec-sync, the largest: change audit 4.9 s against 6.3.0's 5.0 s; a root-specs walk there would cover 1,901 directories outside .git and .specsync (find: 0.15 s). fledge lanes run verify passed at 967e6bdb (3107 unit, 630 integration, 62/62 specs, 100% coverage); change audit --strict exit 0; pre-push and fledge trust verify passed.**

Recorded as passed in local macOS 26, rust 1.89.0, branch 1081-vanished-requirements at 967e6bdb at revision 967e6bdb34dd, observed 2026-11-19T16:16:02Z.
