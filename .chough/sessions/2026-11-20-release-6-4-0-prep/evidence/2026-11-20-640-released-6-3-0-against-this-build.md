---
kind: evidence
id: 2026-11-20-640-released-6-3-0-against-this-build
aliases: []
claim: "Homebrew's released specsync 6.3.0 against main 10d9bb3f's release build (tree 884d3eba, which printed 6.3.0 before the bump) on one PR-shaped fixture: 6.3.0 check passes at every tip; 6.3.0 change audit and audit --strict exit 1 on the branch that adds a history fragment, at the checked and the archive tip, and pass on main after the merge; 6.3.0 change check --commit on a change the new build recorded commits a Change Log row and version 2, the new check then exits 1 naming the double record, and removing the row (then restoring version:) makes check pass, after which change check --commit and audit --strict pass; approve --dry-run --json prints id, dry_run, would_approve and thirteen checks. The upgrade notes and the corrected #1084 CHANGELOG sentence quote this run."
result: passed
evidence_kind: product-test
revision: 7ff79527093faf02d6d9d9370c673502f56e8501
content: 016054af00bb40e433cbc1ee4239b79fe0d437d5b8f2b4fd5bd7ed00a25b44df
environment: macOS 27.2 (Darwin 27.2.0), Apple M5 Ultra, local clone
artifacts:
  - .chough/sessions/2026-11-20-release-6-4-0-prep/evidence/2026-11-20-6-3-0-against-6-4-0-fixture.log
  - .chough/sessions/2026-11-20-release-6-4-0-prep/evidence/2026-11-20-6-3-0-against-6-4-0-fixture.sh
artifact_fingerprints:
  - path: .chough/sessions/2026-11-20-release-6-4-0-prep/evidence/2026-11-20-6-3-0-against-6-4-0-fixture.log
    digest: 996a4a71d29dbad9969d1da00c921572bc2167e6084e4804e3279a86d1978231
  - path: .chough/sessions/2026-11-20-release-6-4-0-prep/evidence/2026-11-20-6-3-0-against-6-4-0-fixture.sh
    digest: 4926ca74d1278df67e5427f96d8b5fc662542e456e32f8629a21bc5e6b7d96ba
criteria: [CI-1.a, DOCS-4]
requirements: []
contracts: []
limitations: []
observed: 2026-11-20T01:39:21Z
actor:
  - agent: claude-code
    terminal: t59
---
**Homebrew's released specsync 6.3.0 against main 10d9bb3f's release build (tree 884d3eba, which printed 6.3.0 before the bump) on one PR-shaped fixture: 6.3.0 check passes at every tip; 6.3.0 change audit and audit --strict exit 1 on the branch that adds a history fragment, at the checked and the archive tip, and pass on main after the merge; 6.3.0 change check --commit on a change the new build recorded commits a Change Log row and version 2, the new check then exits 1 naming the double record, and removing the row (then restoring version:) makes check pass, after which change check --commit and audit --strict pass; approve --dry-run --json prints id, dry_run, would_approve and thirteen checks. The upgrade notes and the corrected #1084 CHANGELOG sentence quote this run.**

Recorded as passed in macOS 27.2 (Darwin 27.2.0), Apple M5 Ultra, local clone at revision 7ff79527093f, observed 2026-11-20T01:39:21Z.
