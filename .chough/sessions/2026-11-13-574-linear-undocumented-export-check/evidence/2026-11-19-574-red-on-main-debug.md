---
kind: evidence
id: 2026-11-19-574-red-on-main-debug
aliases: []
claim: "Red control for the resized timing test (32,000 against 128,000 undocumented exports, interleaved, fastest of three), on main 3bebe8b0 with only the test added, idle machine, debug build: 14.9 times as long (7.06s, 104.9s); linear is about 4 and the bound is 7. This is the defect #574 reports, fixed by this change; not a new bug."
result: failed
evidence_kind: product-test
revision: 76a6603257a1efec6ba25d83c603617ce7434c46
content: f0217faea17b65918e339194d4f4b1f32de578d356b50664d93f1e0fd5b2eab3
environment: macOS 27.2 (Darwin 27.2.0), Apple M5 Ultra, local clone
fingerprint: sha256:805e38d8683e6d48229b1a14f3bd89bcb0a73e4c6bd39319033c302531d38b07
external: ext-978ca9
criteria: [SPEED-1.b]
requirements: []
contracts: [specs/validator/validator.spec.md]
limitations: []
observed: 2026-11-19T08:42:55Z
actor:
  - agent: claude-code
    terminal: t59
---
**Red control for the resized timing test (32,000 against 128,000 undocumented exports, interleaved, fastest of three), on main 3bebe8b0 with only the test added, idle machine, debug build: 14.9 times as long (7.06s, 104.9s); linear is about 4 and the bound is 7. This is the defect #574 reports, fixed by this change; not a new bug.**

Recorded as failed in macOS 27.2 (Darwin 27.2.0), Apple M5 Ultra, local clone at revision 76a6603257a1, observed 2026-11-19T08:42:55Z.
