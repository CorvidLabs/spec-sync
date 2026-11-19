---
kind: evidence
id: 2026-11-13-at-3810ab929-fix-887-with-origin-main-merged-in-the-verify
aliases: []
claim: "At 3810ab929 (fix/887 with origin/main merged in), the verify lane's cargo test failed one test: cold_archived_status_prefetch_workers_reuse_repository_facts, 237 Git processes with six archives against main's 222, because a show <rev>:<path> of a path the revision does not hold was no longer memoized. Fixed in 1898dc4bb in this session by keeping the batch reader's missing answer, and Git's 'does not exist in', as definite; found and fixed before review, so no bug record."
result: failed
evidence_kind: product-test
revision: 1898dc4bb4b2d11bbe740b52a94493b150ad50f9
content: ded734ebf635232c49a9d58035421dcccab754fb30318f9780b063175a205127
environment: macOS 26 (Darwin 25.5.0), Apple M1 Ultra, local worktree
fingerprint: sha256:73ed0377a62132a9177274536ef2c5ca58f0c66292ff7637c680bc793705dbd6
external: ext-487c6e
criteria: [SPEED-1.c]
requirements: []
contracts: []
limitations: []
observed: 2026-11-13T21:09:04Z
actor:
  - agent: claude-code
    terminal: t38p
---
**At 3810ab929 (fix/887 with origin/main merged in), the verify lane's cargo test failed one test: cold_archived_status_prefetch_workers_reuse_repository_facts, 237 Git processes with six archives against main's 222, because a show <rev>:<path> of a path the revision does not hold was no longer memoized. Fixed in 1898dc4bb in this session by keeping the batch reader's missing answer, and Git's 'does not exist in', as definite; found and fixed before review, so no bug record.**

Recorded as failed in macOS 26 (Darwin 25.5.0), Apple M1 Ultra, local worktree at revision 1898dc4bb4b2, observed 2026-11-13T21:09:04Z.
