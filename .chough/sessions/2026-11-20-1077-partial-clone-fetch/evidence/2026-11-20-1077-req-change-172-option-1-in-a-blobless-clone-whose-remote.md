---
kind: evidence
id: 2026-11-20-1077-req-change-172-option-1-in-a-blobless-clone-whose-remote
aliases: []
claim: "#1077 (REQ-change-172 option 1), in a blobless clone whose remote resolves only through the user's global insteadOf: change status, audit and ship-status succeed after one fetch announced by one line, and nothing under .specsync and specs is missing afterwards; a full clone makes no fetch; with the user's configuration gone the refusal says the clone is partial and names a git -C … fetch … <oid> … that, run as printed, lets the command pass; a blobless clone on main audits #1081's base-commit requirements after the one fetch. Real repos (rule 3): the blobless HTTPS clone of private peck: this build downloads 377 objects once and then gives the full clone's 85 errors with no 'cannot tell' (the build before gave 11). Added path src/change/validate_loaded_change.rs re-approved under Leif's standing OK (t4c, about 06:04Z). Lane: fledge lanes run verify passed at c783e8cf through one-lane (3137 unit, 654 integration, 62/62 specs, 100% coverage); change audit --strict exit 0. Red first f98998f3 (through the binary, tests/integration/git_environments.rs, shapes made by the test so nothing relies on a Git default); fix 850317cd, 6ea81ef2, 1cbe69cb."
result: passed
evidence_kind: product-test
revision: 349d40236b99730f4509a4559ff95fcfe7e861ab
content: 956d73e9c39c4f15679f69f6f14f374c23eaad5b00edc19a55feadf27f83018b
environment: local macOS 26, rust 1.89.0, branch 909-1072-1077-git-environments at c783e8cf
criteria: [CHANGE-7]
requirements: []
contracts: []
limitations: []
observed: 2026-11-20T06:14:58Z
actor:
  - agent: claude-code
    terminal: t4t
---
**#1077 (REQ-change-172 option 1), in a blobless clone whose remote resolves only through the user's global insteadOf: change status, audit and ship-status succeed after one fetch announced by one line, and nothing under .specsync and specs is missing afterwards; a full clone makes no fetch; with the user's configuration gone the refusal says the clone is partial and names a git -C … fetch … <oid> … that, run as printed, lets the command pass; a blobless clone on main audits #1081's base-commit requirements after the one fetch. Real repos (rule 3): the blobless HTTPS clone of private peck: this build downloads 377 objects once and then gives the full clone's 85 errors with no 'cannot tell' (the build before gave 11). Added path src/change/validate_loaded_change.rs re-approved under Leif's standing OK (t4c, about 06:04Z). Lane: fledge lanes run verify passed at c783e8cf through one-lane (3137 unit, 654 integration, 62/62 specs, 100% coverage); change audit --strict exit 0. Red first f98998f3 (through the binary, tests/integration/git_environments.rs, shapes made by the test so nothing relies on a Git default); fix 850317cd, 6ea81ef2, 1cbe69cb.**

Recorded as passed in local macOS 26, rust 1.89.0, branch 909-1072-1077-git-environments at c783e8cf at revision 349d40236b99, observed 2026-11-20T06:14:58Z.
