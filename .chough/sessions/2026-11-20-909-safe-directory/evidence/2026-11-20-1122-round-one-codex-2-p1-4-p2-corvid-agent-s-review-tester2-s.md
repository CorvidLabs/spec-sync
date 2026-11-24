---
kind: evidence
id: 2026-11-20-1122-round-one-codex-2-p1-4-p2-corvid-agent-s-review-tester2-s
aliases: []
claim: "#1122 round one (Codex 2 P1 + 4 P2, corvid-agent's review, tester2's Linux run). Red first da1b8d91 and fd7f1aa3, fixes 647c58af and 7efd6e73: every history read in run_git_bounded triggers the one partial-clone fetch (audit with no active change: one fetch); the trust probe and the fetch are rooted (GIT_DIR and the rest removed) and the probe counts only for this repository's top level; stderr bounded and never joined, the fetch's session killed on timeout; gitdir and commondir in the owner check and the file's placement; exact top-level bytes; Git's boolean spellings; '--' before the remote; trust tests set GIT_CONFIG_NOSYSTEM. Lane at 7ad4ff5d through one-lane (3137 unit, 660 integration, 62/62, 100%); change audit --strict exit 0. Real repos: spec-sync fresh blobless audit 42.6 s (6.3.0) against 2.3 s with one fetch of 3245 objects; private peck blobless 85 errors as before after one fetch of 377 objects; GIT_DIR naming a trusted sibling still refuses."
result: passed
evidence_kind: product-test
revision: 7ad4ff5d9b45d3df182c6e6a859b5a14ceecf3c1
content: 53059ed86c4fa1ee97127c73b856c4d6dedf09cc7f3df48dd1f1001080106857
environment: local macOS 26, rust 1.89.0, branch 909-1072-1077-git-environments at 7ad4ff5d
criteria: [CHANGE-7]
requirements: []
contracts: []
limitations: []
observed: 2026-11-20T12:23:20Z
actor:
  - agent: claude-code
    terminal: t4t
---
**#1122 round one (Codex 2 P1 + 4 P2, corvid-agent's review, tester2's Linux run). Red first da1b8d91 and fd7f1aa3, fixes 647c58af and 7efd6e73: every history read in run_git_bounded triggers the one partial-clone fetch (audit with no active change: one fetch); the trust probe and the fetch are rooted (GIT_DIR and the rest removed) and the probe counts only for this repository's top level; stderr bounded and never joined, the fetch's session killed on timeout; gitdir and commondir in the owner check and the file's placement; exact top-level bytes; Git's boolean spellings; '--' before the remote; trust tests set GIT_CONFIG_NOSYSTEM. Lane at 7ad4ff5d through one-lane (3137 unit, 660 integration, 62/62, 100%); change audit --strict exit 0. Real repos: spec-sync fresh blobless audit 42.6 s (6.3.0) against 2.3 s with one fetch of 3245 objects; private peck blobless 85 errors as before after one fetch of 377 objects; GIT_DIR naming a trusted sibling still refuses.**

Recorded as passed in local macOS 26, rust 1.89.0, branch 909-1072-1077-git-environments at 7ad4ff5d at revision 7ad4ff5d9b45, observed 2026-11-20T12:23:20Z.
