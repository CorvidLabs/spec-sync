---
kind: evidence
id: 2026-11-13-red-first-control-on-main-18385ef5e-with-only-the-new-test-git
aliases: []
claim: "Red-first control: on main 18385ef5e with only the new test, git_utils::tests::a_failed_fact_is_not_memoized_but_a_definite_absence_is fails: a rev-parse that failed with 128 was memoized. The failure is the defect #887 reports, fixed by this change; not a new bug."
result: failed
evidence_kind: product-test
revision: 1898dc4bb4b2d11bbe740b52a94493b150ad50f9
content: ded734ebf635232c49a9d58035421dcccab754fb30318f9780b063175a205127
environment: macOS 26 (Darwin 25.5.0), Apple M1 Ultra, local worktree
fingerprint: sha256:bed5490dde41f625a35a5af56e789925792e09650ebe4ca9f11f2d8187b8ba41
external: ext-e641b6
criteria: [SPEED-1.c]
requirements: []
contracts: []
limitations: []
observed: 2026-11-13T21:09:06Z
actor:
  - agent: claude-code
    terminal: t38p
---
**Red-first control: on main 18385ef5e with only the new test, git_utils::tests::a_failed_fact_is_not_memoized_but_a_definite_absence_is fails: a rev-parse that failed with 128 was memoized. The failure is the defect #887 reports, fixed by this change; not a new bug.**

Recorded as failed in macOS 26 (Darwin 25.5.0), Apple M1 Ultra, local worktree at revision 1898dc4bb4b2, observed 2026-11-13T21:09:06Z.
