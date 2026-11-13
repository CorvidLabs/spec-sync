---
kind: evidence
id: 2026-11-13-red-control-weaker-than-the-others-the-red-commit-665f8a220
aliases: []
claim: "Red control, weaker than the others: the red commit 665f8a220 does not compile, because a_repository_fact_seed_shares_one_copy_of_its_facts_across_workers uses the shared-copy seed API this change introduces. No behavioural failure on the old code was demonstrated; the test pins the new sharing (one Arc copy across workers)."
result: failed
evidence_kind: structural
revision: 2915d230bb7fdfd75aebe1c9e04f29156d6470e4
content: 62d7d5d67722539088101d89a2dfce6bd37113bb32be262a6a40b90e904d845c
environment: macOS 26 (Darwin 25.5.0), Apple M1 Ultra, local worktree
fingerprint: sha256:e0b1c7db7f389ed4cbfa678f3e3d098a29ca293b5029247fb9ae6f4861ce818b
external: ext-ab6578
criteria: [SPEED-1.a]
requirements: []
contracts: []
limitations: []
observed: 2026-11-13T20:08:19Z
actor:
  - agent: claude-code
    terminal: t38p
---
**Red control, weaker than the others: the red commit 665f8a220 does not compile, because a_repository_fact_seed_shares_one_copy_of_its_facts_across_workers uses the shared-copy seed API this change introduces. No behavioural failure on the old code was demonstrated; the test pins the new sharing (one Arc copy across workers).**

Recorded as failed in macOS 26 (Darwin 25.5.0), Apple M1 Ultra, local worktree at revision 2915d230bb7f, observed 2026-11-13T20:08:19Z.
