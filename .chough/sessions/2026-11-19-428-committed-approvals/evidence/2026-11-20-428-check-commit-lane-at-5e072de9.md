---
kind: evidence
id: 2026-11-20-428-check-commit-lane-at-5e072de9
aliases: []
claim: "Leif's 'Yes (Recommended)' (2026-11-20 about 14:13Z): REQ-change-166 adds that change check --commit refuses, before writing anything, to commit a ledger whose working copy changes or drops a committed record, naming the same restore, and its criterion. run_checked_commit asks it before the first pass, for the change given or the one change check would select. Checked at 5e072de9: fledge lanes run verify 10/10 in 9m56s through one-lane (unit 3127, 2 ignored; integration 647; spec check 62/62, 168/168 files); change audit --strict passes; the requirement text differs from b7de737c by exactly the two insertions (by script)."
result: passed
evidence_kind: product-test
revision: 5e072de927bc0fd89f65ddf7beb771c72995f155
content: 18656c5769b54c91ef3bf863246291b74a23f58f679f8fe1ae53afce115ac020
environment: macOS 27.2 (Darwin 27.2.0), Apple M5 Ultra, local clone
fingerprint: sha256:f3e51851b900f340d1057d6ef535092ae38dd4db7b101b2d8dd3af11124f001f
external: ext-58e249
criteria: [CHANGE-16]
requirements: []
contracts: [specs/change/change.spec.md, specs/cmd_change/cmd_change.spec.md]
limitations: []
observed: 2026-11-20T14:46:34Z
actor:
  - agent: claude-code
    terminal: t59
---
**Leif's 'Yes (Recommended)' (2026-11-20 about 14:13Z): REQ-change-166 adds that change check --commit refuses, before writing anything, to commit a ledger whose working copy changes or drops a committed record, naming the same restore, and its criterion. run_checked_commit asks it before the first pass, for the change given or the one change check would select. Checked at 5e072de9: fledge lanes run verify 10/10 in 9m56s through one-lane (unit 3127, 2 ignored; integration 647; spec check 62/62, 168/168 files); change audit --strict passes; the requirement text differs from b7de737c by exactly the two insertions (by script).**

Recorded as passed in macOS 27.2 (Darwin 27.2.0), Apple M5 Ultra, local clone at revision 5e072de927bc, observed 2026-11-20T14:46:34Z.
