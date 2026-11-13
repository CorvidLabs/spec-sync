---
kind: evidence
id: 2026-11-13-red-first-control-against-main-s-code-the-three-lifecycle-file
aliases: []
claim: "Red-first control: against main's code the three lifecycle_file_errors tests fail: a type error in approvals.json did not name the file, and state.json and verification-attempts.json from a newer SpecSync were read or not refused alike. These are the gaps #434 reports, fixed by this change; not new bugs."
result: failed
evidence_kind: product-test
revision: 791158b3e0981a215936c139dbe69a848db28015
content: 0b2cb41ba5b6e1276780b62cacf0fa017c89fd1abd85caa101fa2990ff2a1f36
environment: macOS 26 (Darwin 25.5.0), Apple M1 Ultra, local worktree
fingerprint: sha256:ba23f9e69a5dc1796c448f173e8a62fa2d354ca4a8bd661bdb889f3693b4133d
external: ext-a3b6e9
criteria: [CHECK-9]
requirements: [REQ-change-164, REQ-change-165]
contracts: []
limitations: []
observed: 2026-11-13T20:55:11Z
actor:
  - agent: claude-code
    terminal: t38p
---
**Red-first control: against main's code the three lifecycle_file_errors tests fail: a type error in approvals.json did not name the file, and state.json and verification-attempts.json from a newer SpecSync were read or not refused alike. These are the gaps #434 reports, fixed by this change; not new bugs.**

Recorded as failed in macOS 26 (Darwin 25.5.0), Apple M1 Ultra, local worktree at revision 791158b3e098, observed 2026-11-13T20:55:11Z.
