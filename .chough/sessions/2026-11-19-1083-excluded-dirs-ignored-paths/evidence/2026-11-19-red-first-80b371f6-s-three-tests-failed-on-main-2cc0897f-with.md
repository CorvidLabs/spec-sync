---
kind: evidence
id: 2026-11-19-red-first-80b371f6-s-three-tests-failed-on-main-2cc0897f-with
aliases: []
claim: "Red first: 80b371f6's three tests failed on main 2cc0897f with NFTMix's own message ('.chough/sessions/ overlaps source_dirs .'). The fix is c87762e8: the source_dirs overlap skips an entry with a component below the root exactly equal to an exclude_dirs name. Now 28 tests in the file pass: the NFTMix shape accepted; .Chough/sessions/ and .chough/tools/cache/ under source_dirs ['.chough/tools'] refused; .specsync, the specs tree, files:, principles file, template and meaningful_paths each still named under an excluded name; a root-code project approves, verifies, stays current with records written under the three folders, and lists them; every earlier test unchanged. Real repo: CorvidLabs/NFTMix e60625a with .agents and .chough added to exclude_dirs and the three Chough folders listed: 6.3.0 refuses the policy in change new and audit; this build accepts it (change new rc 0, status lists the three under Verification ignores, coverage 100%). NFTMix's audit still fails on its own records, the same under both builds apart from the policy: CHG-0002 is accepted but unarchived with affected_paths ['.'], which the existing per-change rule refuses, and CHG-0001 has an incomplete artifact. fledge lanes run verify passed at a218d6b6 (3097 unit, 629 integration, 62/62 specs, 100% coverage); change audit --strict exit 0; pre-push and fledge trust verify passed."
result: passed
evidence_kind: product-test
revision: a218d6b62cfde860fa1a0ec3758be1a150737a1b
content: 706515e27aa2a9e045ac77f3ba7fd156bd6485c713c5a1f5aeb54f31b95b59fb
environment: local macOS 26, rust 1.89.0, branch 1083-excluded-dirs-ignored-paths at a218d6b6
criteria: [SETUP-2.b]
requirements: [REQ-change-115]
contracts: []
limitations: []
observed: 2026-11-19T16:46:06Z
actor:
  - agent: claude-code
    terminal: t4t
---
**Red first: 80b371f6's three tests failed on main 2cc0897f with NFTMix's own message ('.chough/sessions/ overlaps source_dirs .'). The fix is c87762e8: the source_dirs overlap skips an entry with a component below the root exactly equal to an exclude_dirs name. Now 28 tests in the file pass: the NFTMix shape accepted; .Chough/sessions/ and .chough/tools/cache/ under source_dirs ['.chough/tools'] refused; .specsync, the specs tree, files:, principles file, template and meaningful_paths each still named under an excluded name; a root-code project approves, verifies, stays current with records written under the three folders, and lists them; every earlier test unchanged. Real repo: CorvidLabs/NFTMix e60625a with .agents and .chough added to exclude_dirs and the three Chough folders listed: 6.3.0 refuses the policy in change new and audit; this build accepts it (change new rc 0, status lists the three under Verification ignores, coverage 100%). NFTMix's audit still fails on its own records, the same under both builds apart from the policy: CHG-0002 is accepted but unarchived with affected_paths ['.'], which the existing per-change rule refuses, and CHG-0001 has an incomplete artifact. fledge lanes run verify passed at a218d6b6 (3097 unit, 629 integration, 62/62 specs, 100% coverage); change audit --strict exit 0; pre-push and fledge trust verify passed.**

Recorded as passed in local macOS 26, rust 1.89.0, branch 1083-excluded-dirs-ignored-paths at a218d6b6 at revision a218d6b62cfd, observed 2026-11-19T16:46:06Z.
