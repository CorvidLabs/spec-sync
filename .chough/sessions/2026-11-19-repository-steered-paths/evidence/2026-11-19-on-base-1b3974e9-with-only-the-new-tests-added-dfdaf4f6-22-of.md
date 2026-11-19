---
kind: evidence
id: 2026-11-19-on-base-1b3974e9-with-only-the-new-tests-added-dfdaf4f6-22-of
aliases: []
claim: "On base 1b3974e9 with only the new tests added (dfdaf4f6): 22 of 24 new integration tests and both new hash_cache unit tests fail (also REQ-cmd-check-020/021, REQ-hash-cache-009/010, REQ-cmd-import-004, REQ-cmd-init-registry-004). change answer and change approve wrote change.md through a committed link out and into a pre-commit hook; change answer created a dangling artifact link's outside target; change new built its workspace in a linked .specsync/changes; check --fix --backup copied into a linked backup-fix and over a linked backup file; init-registry created its registry in a linked .specsync; hooks install appended into a Git hook through CLAUDE.md; a subproject's links that stay in the repository were refused, and an absolute link was not refused as one; a tracked hashes.json was used by check and rewritten by rehash, and a forged pass in it replayed; a configured directory outside the project was not a config error, and a specs link out and a 257-level tree did not stop discovery; changelog gave no skip for an over-bound blob; import --from-dir imported through links out; prune kept keys outside the project; the write scan found 117 direct writes and the read scan 27 direct reads outside the helpers. The two controls (a subproject link out still refused; a backup with no link) pass."
result: failed
evidence_kind: product-test
method: "cargo test (through one-lane) on 1b3974e9 with only the tests commit dfdaf4f6 added: the new integration tests in project_writes, repository_inputs and the read scan, and the two new hash_cache unit tests"
revision: 1b3974e9a025f37ce712d4ad627e99f780d6f207
content: ad2f6ff75fe468442665c78b181119a9c24e61e23f798a6616fb5bf79c690aaf
environment: local-macos-arm64
criteria: [SETUP-6, SETUP-7, CHECK-1.f]
requirements:
  - REQ-util-004
  - REQ-util-005
  - REQ-change-163
  - REQ-config-017
  - REQ-validator-050
  - REQ-changelog-002
contracts: []
limitations: []
observed: 2026-11-19T20:16:29Z
actor:
  - agent: claude-code
    terminal: t5r
---
**On base 1b3974e9 with only the new tests added (dfdaf4f6): 22 of 24 new integration tests and both new hash_cache unit tests fail (also REQ-cmd-check-020/021, REQ-hash-cache-009/010, REQ-cmd-import-004, REQ-cmd-init-registry-004). change answer and change approve wrote change.md through a committed link out and into a pre-commit hook; change answer created a dangling artifact link's outside target; change new built its workspace in a linked .specsync/changes; check --fix --backup copied into a linked backup-fix and over a linked backup file; init-registry created its registry in a linked .specsync; hooks install appended into a Git hook through CLAUDE.md; a subproject's links that stay in the repository were refused, and an absolute link was not refused as one; a tracked hashes.json was used by check and rewritten by rehash, and a forged pass in it replayed; a configured directory outside the project was not a config error, and a specs link out and a 257-level tree did not stop discovery; changelog gave no skip for an over-bound blob; import --from-dir imported through links out; prune kept keys outside the project; the write scan found 117 direct writes and the read scan 27 direct reads outside the helpers. The two controls (a subproject link out still refused; a backup with no link) pass.**

Recorded as failed in local-macos-arm64 at revision 1b3974e9a025, observed 2026-11-19T20:16:29Z.

## Key failing lines

- 117 direct file write(s) bypass the confined project helpers (REQ-util-002); 27 direct file read(s) bypass the bounded project read (REQ-util-003); integration: 2 passed, 22 failed; hash_cache unit: 0 passed, 2 failed
