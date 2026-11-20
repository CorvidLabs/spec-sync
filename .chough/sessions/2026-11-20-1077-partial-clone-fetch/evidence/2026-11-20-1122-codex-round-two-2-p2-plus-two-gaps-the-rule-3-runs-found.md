---
kind: evidence
id: 2026-11-20-1122-codex-round-two-2-p2-plus-two-gaps-the-rule-3-runs-found
aliases: []
claim: "#1122 Codex round two (2 P2) plus two gaps the rule-3 runs found in the same download. Red first fe275bdb, 24d64637 and ed3495d5; fixes 158e3654, f4617e03 and 2917f7d6: the one fetch lists .specsync.toml, specsync.json and specsync-registry.toml beside .specsync; the printed recovery is printf '%s\\n' <oid> ... | git -C <root> fetch ... --stdin -- <remote> (the ids in the line, none in Git's arguments); a failed download gives the loss check's way out as that fetch, not git fetch origin; the listing walks with --full-history. Lane at 7a571bbe through one-lane (3137 unit, 664 integration, 62/62, 100%); change audit --strict exit 0. Real repos, download failing: private peck audit refuses naming 377 ids in a 15.7 KB line, run as printed rc 0, then the same 85 errors as its full clone; spec-sync with its remote unreachable, audit --archives refuses naming 3329 ids in a 137 KB line, run as printed rc 0, then 0 errors as its full clone. With the user's config: spec-sync blobless audit 42.1 s on 6.3.0 against 1.9 s with one fetch of 3329 objects; peck one fetch of 377 then its 85 errors."
result: passed
evidence_kind: product-test
revision: 7a571bbe3ca254f49d51089da6f01a3708c722c8
content: 41452a8b171133f8edaf8b3f99d5505694ce5651d2b29c9c51df8fde6f3c9295
environment: local macOS 26, rust 1.89.0, branch 909-1072-1077-git-environments at 7a571bbe
criteria: [CHANGE-7]
requirements: []
contracts: []
limitations: []
observed: 2026-11-20T13:52:06Z
actor:
  - agent: claude-code
    terminal: t4t
---
**#1122 Codex round two (2 P2) plus two gaps the rule-3 runs found in the same download. Red first fe275bdb, 24d64637 and ed3495d5; fixes 158e3654, f4617e03 and 2917f7d6: the one fetch lists .specsync.toml, specsync.json and specsync-registry.toml beside .specsync; the printed recovery is printf '%s\n' <oid> ... | git -C <root> fetch ... --stdin -- <remote> (the ids in the line, none in Git's arguments); a failed download gives the loss check's way out as that fetch, not git fetch origin; the listing walks with --full-history. Lane at 7a571bbe through one-lane (3137 unit, 664 integration, 62/62, 100%); change audit --strict exit 0. Real repos, download failing: private peck audit refuses naming 377 ids in a 15.7 KB line, run as printed rc 0, then the same 85 errors as its full clone; spec-sync with its remote unreachable, audit --archives refuses naming 3329 ids in a 137 KB line, run as printed rc 0, then 0 errors as its full clone. With the user's config: spec-sync blobless audit 42.1 s on 6.3.0 against 1.9 s with one fetch of 3329 objects; peck one fetch of 377 then its 85 errors.**

Recorded as passed in local macOS 26, rust 1.89.0, branch 909-1072-1077-git-environments at 7a571bbe at revision 7a571bbe3ca2, observed 2026-11-20T13:52:06Z.
