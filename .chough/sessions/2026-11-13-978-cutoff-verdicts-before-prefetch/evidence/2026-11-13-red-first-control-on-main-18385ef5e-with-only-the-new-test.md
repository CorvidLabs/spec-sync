---
kind: evidence
id: 2026-11-13-red-first-control-on-main-18385ef5e-with-only-the-new-test
aliases: []
claim: "Red-first control: on main 18385ef5e with only the new test, change::tests::prefetch_workers_leave_every_cutoff_verdict_in_the_read_cache fails. Workers re-read the cutoff tree and kept nothing in the persistent read cache. The failure is the defect #978 reports, fixed by this change; not a new bug."
result: failed
evidence_kind: product-test
revision: db5f6c2e39c427478737c36f68814001d60126b6
content: 8c91366e70c9b77ad2c48761d56fdc332d386bca92145e82ae7ad0b05bb6daa5
environment: macOS 26 (Darwin 25.5.0), Apple M1 Ultra, local worktree
fingerprint: sha256:40dabc638d7c329139175cc66731d7cbdd7013f2d88b2d258102b5aad4c377b6
external: ext-6c023a
criteria: [SPEED-1.c]
requirements: []
contracts: []
limitations: []
observed: 2026-11-13T21:51:40Z
actor:
  - agent: claude-code
    terminal: t38p
---
**Red-first control: on main 18385ef5e with only the new test, change::tests::prefetch_workers_leave_every_cutoff_verdict_in_the_read_cache fails. Workers re-read the cutoff tree and kept nothing in the persistent read cache. The failure is the defect #978 reports, fixed by this change; not a new bug.**

Recorded as failed in macOS 26 (Darwin 25.5.0), Apple M1 Ultra, local worktree at revision db5f6c2e39c4, observed 2026-11-13T21:51:40Z.
