---
kind: evidence
id: 2026-11-20-657-older-binaries-read-coverage-json
aliases: []
claim: "The change's fifth criterion: released 6.3.0 (Homebrew) and main a28e3042's 6.4.0 build, on a real archive given a committed coverage.json: change status, --json change status (archived, authenticated-history), change audit --strict, change audit --strict --archives, check --strict, change list and doctor all exit 0. This branch's build reads the same archive with the same exit codes."
result: passed
evidence_kind: product-test
revision: c7e5daba2b794e31975863df7ed90d1f2c152e01
content: d94588e6d0f6595361b1100b1fa6c55ef793d3486e3a31e0f12e5c5ddc8c9173
environment: macOS 27.2 (Darwin 27.2.0), Apple M5 Ultra, local clone
fingerprint: sha256:8b35e3d89fae10da89a08590e45c7401f21add17bd52199d3e15bca1de4a7419
external: ext-20dc99
criteria: [SHIP-13.b]
requirements: []
contracts: []
limitations: []
observed: 2026-11-20T07:30:03Z
actor:
  - agent: claude-code
    terminal: t59
---
**The change's fifth criterion: released 6.3.0 (Homebrew) and main a28e3042's 6.4.0 build, on a real archive given a committed coverage.json: change status, --json change status (archived, authenticated-history), change audit --strict, change audit --strict --archives, check --strict, change list and doctor all exit 0. This branch's build reads the same archive with the same exit codes.**

Recorded as passed in macOS 27.2 (Darwin 27.2.0), Apple M5 Ultra, local clone at revision c7e5daba2b79, observed 2026-11-20T07:30:03Z.
