---
kind: evidence
id: 2026-11-20-1104-red-shared-constant-at-31b3a26a
aliases: []
claim: "Red first at 31b3a26a: the_successor_search_reads_the_words_both_writers_write fails to compile, E0425 cannot find value CHANGED_AFTER_ACCEPTANCE, before the shared constant exists; everything else in the test compiles. cargo test --bin specsync through one-lane, the neutral git config as a prefix only (#1092)."
result: failed
evidence_kind: product-test
revision: 880dedcbf94ccd8d8e88ae9d786d42b38c45044c
content: 5cafb6cceb18a3f12164612296a31f68ffcd279cf1dff0b68ef5ad8f45bdbc6f
environment: macOS 27.2 (Darwin 27.2.0), Apple M5 Ultra, local clone
fingerprint: sha256:33e73e24dcb3d5a758206c08b9362f6aa016e0de88d7957556b6d53f17585621
external: ext-ff473f
criteria: [CHANGE-7]
requirements: []
contracts: [specs/change/change.spec.md]
limitations: []
observed: 2026-11-20T03:38:39Z
actor:
  - agent: claude-code
    terminal: t59
---
**Red first at 31b3a26a: the_successor_search_reads_the_words_both_writers_write fails to compile, E0425 cannot find value CHANGED_AFTER_ACCEPTANCE, before the shared constant exists; everything else in the test compiles. cargo test --bin specsync through one-lane, the neutral git config as a prefix only (#1092).**

Recorded as failed in macOS 27.2 (Darwin 27.2.0), Apple M5 Ultra, local clone at revision 880dedcbf94c, observed 2026-11-20T03:38:39Z.
