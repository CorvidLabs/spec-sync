---
kind: evidence
id: 2026-11-20-on-4516f355-with-only-change-8-s-tests-added-all-6-new-unit
aliases: []
claim: "On 4516f355 with only change 8's tests added: all 6 new unit tests and the new scaffold CLI test fail (also REQ-validator-053, REQ-config-018, REQ-schema-003, REQ-commands-016, REQ-cmd-scaffold-008). change adopt's requirement proposals listed two requirements.md files behind a specs link out; the duplicate requirement ID check named specs/alpha and specs/beta behind it; source detection found lib behind a link out; the schema snapshot named outside_only.sql; a missing file's suggestion listed ../outside/data; scaffold refused auth over an Auth folder outside; the read cache path was under .git."
result: failed
evidence_kind: product-test
revision: 38cc5e879a4c890a838368a50138a9135e6893f7
content: c661cc4880af0229c6efb480fd25ae19f34619e9dba3aa647afdf7c94e5cbb6c
environment: macOS arm64 (APFS), rustc 1.89.0, isolated git config, through one-lane; tree of 3dc6aff3 and 38cc5e87 (4516f355 + change 8's records and tests only)
fingerprint: sha256:52ed1bf2be6905f7017b60503dacabd2b57e88740028bd74c30899da8988f995
external: ext-ad29ef
criteria: [SETUP-6, SETUP-7, CHECK-1.f]
requirements: [REQ-util-009, REQ-util-010, REQ-change-164]
contracts: []
limitations: []
observed: 2026-11-20T04:56:09Z
actor:
  - agent: claude-code
    terminal: t5r
---
**On 4516f355 with only change 8's tests added: all 6 new unit tests and the new scaffold CLI test fail (also REQ-validator-053, REQ-config-018, REQ-schema-003, REQ-commands-016, REQ-cmd-scaffold-008). change adopt's requirement proposals listed two requirements.md files behind a specs link out; the duplicate requirement ID check named specs/alpha and specs/beta behind it; source detection found lib behind a link out; the schema snapshot named outside_only.sql; a missing file's suggestion listed ../outside/data; scaffold refused auth over an Auth folder outside; the read cache path was under .git.**

Recorded as failed in macOS arm64 (APFS), rustc 1.89.0, isolated git config, through one-lane; tree of 3dc6aff3 and 38cc5e87 (4516f355 + change 8's records and tests only) at revision 38cc5e879a4c, observed 2026-11-20T04:56:09Z.
