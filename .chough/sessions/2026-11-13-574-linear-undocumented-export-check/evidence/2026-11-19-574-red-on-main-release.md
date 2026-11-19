---
kind: evidence
id: 2026-11-19-574-red-on-main-release
aliases: []
claim: "Red control for the resized timing test (32,000 against 128,000 undocumented exports, interleaved, fastest of three), on main 3bebe8b0 with only the test added, idle machine, release build: 14.3 times as long (0.56s, 7.95s); linear is about 4 and the bound is 7. This is the defect #574 reports, fixed by this change; not a new bug. The first sizes, 8,000 against 32,000, were red in debug only (the tester measured 5.2 times in release)."
result: failed
evidence_kind: product-test
revision: 76a6603257a1efec6ba25d83c603617ce7434c46
content: f0217faea17b65918e339194d4f4b1f32de578d356b50664d93f1e0fd5b2eab3
environment: macOS 27.2 (Darwin 27.2.0), Apple M5 Ultra, local clone
fingerprint: sha256:34aed61710ac0ed18db68ec11bcd056f7cc2dc2d6ef2cb0c42ceaf8e2181d4b6
external: ext-62e7fc
criteria: [SPEED-1.b]
requirements: []
contracts: [specs/validator/validator.spec.md]
limitations: []
observed: 2026-11-19T08:42:55Z
actor:
  - agent: claude-code
    terminal: t59
---
**Red control for the resized timing test (32,000 against 128,000 undocumented exports, interleaved, fastest of three), on main 3bebe8b0 with only the test added, idle machine, release build: 14.3 times as long (0.56s, 7.95s); linear is about 4 and the bound is 7. This is the defect #574 reports, fixed by this change; not a new bug. The first sizes, 8,000 against 32,000, were red in debug only (the tester measured 5.2 times in release).**

Recorded as failed in macOS 27.2 (Darwin 27.2.0), Apple M5 Ultra, local clone at revision 76a6603257a1, observed 2026-11-19T08:42:55Z.
