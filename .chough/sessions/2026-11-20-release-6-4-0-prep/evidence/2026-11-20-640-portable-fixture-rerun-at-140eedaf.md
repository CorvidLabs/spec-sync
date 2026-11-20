---
kind: evidence
id: 2026-11-20-640-portable-fixture-rerun-at-140eedaf
aliases: []
claim: "One rerun of the fixture after its version restore stopped using BSD-only sed -i '' (it now writes through a temporary file, so GNU sed runs it too), with this branch's own release build (specsync 6.4.0, built at 68b76f27) against Homebrew's released 6.3.0: the same 26 commands and exit codes as the earlier run with main 10d9bb3f's build, which corvid-agent also matched on Linux. The corrected deltas.md sentence quotes this run."
result: passed
evidence_kind: product-test
revision: fe41d4795ba501ed6331e517c3809a3f44bf8359
content: e700eb9394e760976c9af30a1163ebac73c36f98f7b820875be14c8f3581dce2
environment: macOS 27.2 (Darwin 27.2.0), Apple M5 Ultra, local clone
artifacts:
  - .chough/sessions/2026-11-20-release-6-4-0-prep/evidence/2026-11-20-6-3-0-against-6-4-0-fixture.log
  - .chough/sessions/2026-11-20-release-6-4-0-prep/evidence/2026-11-20-6-3-0-against-6-4-0-fixture.sh
artifact_fingerprints:
  - path: .chough/sessions/2026-11-20-release-6-4-0-prep/evidence/2026-11-20-6-3-0-against-6-4-0-fixture.log
    digest: 5af9212735e1e0da126b7f53b513947c4e49717400d2b043f144daa8fcb5410e
  - path: .chough/sessions/2026-11-20-release-6-4-0-prep/evidence/2026-11-20-6-3-0-against-6-4-0-fixture.sh
    digest: 4052920f21bfc564c584d96f75411676a53006d407c874fdd3b3e020f2cb3f1d
criteria: [CI-1.a, DOCS-4]
requirements: []
contracts: []
limitations: []
observed: 2026-11-20T02:13:50Z
actor:
  - agent: claude-code
    terminal: t59
---
**One rerun of the fixture after its version restore stopped using BSD-only sed -i '' (it now writes through a temporary file, so GNU sed runs it too), with this branch's own release build (specsync 6.4.0, built at 68b76f27) against Homebrew's released 6.3.0: the same 26 commands and exit codes as the earlier run with main 10d9bb3f's build, which corvid-agent also matched on Linux. The corrected deltas.md sentence quotes this run.**

Recorded as passed in macOS 27.2 (Darwin 27.2.0), Apple M5 Ultra, local clone at revision fe41d4795ba5, observed 2026-11-20T02:13:50Z.
