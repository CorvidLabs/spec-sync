---
kind: evidence
id: 2026-11-20-corvid-agent-s-review-leif-s-a-and-codex-s-second-round-on
aliases: []
claim: "corvid-agent's review, Leif's 'A' and Codex's second round on #1108, then main a28e3042 (6.4.0) merged in. (1) A dangling origin/HEAD names no default branch (red 652133e2, fix b607951c); an archive record not yet committed accounts for its removals. (2) Leif's 'A' (2026-11-20 about 02:15Z): REQ-change-169 sent back and re-approved with ss-lead's two insertions (904aa694); origin/HEAD, origin/main or origin/master counts only when it resolves and is not the branch HEAD is on, so work directly on main compares from the base commit (red b41aa3fb, fix ac08f7b1). (3) Codex round 2: a file canonical at the reference is judged by the reference's exclusions (red 14d5c323, fix 9293ee73); registry paths outside the project add nothing, a registered spec's companions are its own directory, the restore command is quoted, and the audit asks it of a draft (red b41aa3fb, fix ac08f7b1). (4) The spawn-ceiling test failed on b6869d9a (check 157 over 121): the lookup's symbolic-ref shape dropped memoized repository facts; fixed in fd34edfd (check 116, finalize 294). Merge 14c9928b: no conflicts; the CHANGELOG entry moved under [Unreleased]. Tests are set with git symbolic-ref, so they do not depend on the Git version. Real repos (rule 3): same results as 6.3.0 on spec-sync main a28e3042, Chough main and PRs #311 and #310, starling, NFTMix and a full clone of peck; starling PR #43 still refused with 26 ids; in a blobless partial clone of peck on main, 11 changes now report cannot tell (their base commits' requirement blobs are not local; #1077's shape). fledge lanes run verify passed at 7df83230 through one-lane (3130 unit, 645 integration, 62/62, 100%); change audit --strict exit 0; pre-push and fledge trust verify passed."
result: passed
evidence_kind: product-test
revision: 7df83230e86b7fadf378695261cac7e9a03d99aa
content: 792b639ef86cf17209d51385b436f7577e81bb33d62aab870e6442827f493dfd
environment: local macOS 26, rust 1.89.0, branch 1081-vanished-requirements at 7df83230
criteria: [CHANGE-17, CHANGE-17.a, CHANGE-17.b, CHANGE-17.c]
requirements: [REQ-change-169]
contracts: []
limitations: []
observed: 2026-11-20T03:19:38Z
actor:
  - agent: claude-code
    terminal: t4t
---
**corvid-agent's review, Leif's 'A' and Codex's second round on #1108, then main a28e3042 (6.4.0) merged in. (1) A dangling origin/HEAD names no default branch (red 652133e2, fix b607951c); an archive record not yet committed accounts for its removals. (2) Leif's 'A' (2026-11-20 about 02:15Z): REQ-change-169 sent back and re-approved with ss-lead's two insertions (904aa694); origin/HEAD, origin/main or origin/master counts only when it resolves and is not the branch HEAD is on, so work directly on main compares from the base commit (red b41aa3fb, fix ac08f7b1). (3) Codex round 2: a file canonical at the reference is judged by the reference's exclusions (red 14d5c323, fix 9293ee73); registry paths outside the project add nothing, a registered spec's companions are its own directory, the restore command is quoted, and the audit asks it of a draft (red b41aa3fb, fix ac08f7b1). (4) The spawn-ceiling test failed on b6869d9a (check 157 over 121): the lookup's symbolic-ref shape dropped memoized repository facts; fixed in fd34edfd (check 116, finalize 294). Merge 14c9928b: no conflicts; the CHANGELOG entry moved under [Unreleased]. Tests are set with git symbolic-ref, so they do not depend on the Git version. Real repos (rule 3): same results as 6.3.0 on spec-sync main a28e3042, Chough main and PRs #311 and #310, starling, NFTMix and a full clone of peck; starling PR #43 still refused with 26 ids; in a blobless partial clone of peck on main, 11 changes now report cannot tell (their base commits' requirement blobs are not local; #1077's shape). fledge lanes run verify passed at 7df83230 through one-lane (3130 unit, 645 integration, 62/62, 100%); change audit --strict exit 0; pre-push and fledge trust verify passed.**

Recorded as passed in local macOS 26, rust 1.89.0, branch 1081-vanished-requirements at 7df83230 at revision 7df83230e86b, observed 2026-11-20T03:19:38Z.
