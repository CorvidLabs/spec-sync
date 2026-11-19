---
kind: evidence
id: 2026-11-19-1038-the-real-archive-reopen-and-re-ship-case
aliases: []
claim: "The real case of #1038, driven through the release binary built at e54bc636 on a throwaway project: a change approved, checked, reviewed and shipped; its delivered file edited; change reopen; check, review and ship again. The re-archive tip is 14 renames into the archive, 2 additions there (finalization.json, verification.json) and 2 deletions from the package, with no modified archive file. CI's classify-ci-paths.sh in name-status mode says archive_only=true, and ship-status says tip_class archive_only, review_currency current, no blockers. The first archive tip agrees too."
result: passed
evidence_kind: product-test
method: manual
revision: 5f5cdf8d41eb8ca4e8f3c3b229e8c5f24f10c5c7
content: bf638a95bab9091bab5a1880c42b353e868c47abad1ea4614ebc0ac89c46416e
environment: macOS 27.2 (Darwin 27.2.0), Apple M5 Ultra, local clone
criteria: [SHIP-8]
requirements: []
contracts: [specs/cmd_change/cmd_change.spec.md]
limitations: []
observed: 2026-11-19T06:20:37Z
actor:
  - agent: claude-code
    terminal: t59
---
**The real case of #1038, driven through the release binary built at e54bc636 on a throwaway project: a change approved, checked, reviewed and shipped; its delivered file edited; change reopen; check, review and ship again. The re-archive tip is 14 renames into the archive, 2 additions there (finalization.json, verification.json) and 2 deletions from the package, with no modified archive file. CI's classify-ci-paths.sh in name-status mode says archive_only=true, and ship-status says tip_class archive_only, review_currency current, no blockers. The first archive tip agrees too.**

Recorded as passed in macOS 27.2 (Darwin 27.2.0), Apple M5 Ultra, local clone at revision 5f5cdf8d41eb, observed 2026-11-19T06:20:37Z.
