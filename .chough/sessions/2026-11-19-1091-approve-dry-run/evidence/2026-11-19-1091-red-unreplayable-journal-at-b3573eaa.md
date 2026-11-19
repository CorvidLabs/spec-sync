---
kind: evidence
id: 2026-11-19-1091-red-unreplayable-journal-at-b3573eaa
aliases: []
claim: "Red-first, for the tester's finding on 37496bdf: at b3573eaa (the test alone) approve_dry_run::the_dry_run_gives_the_approvals_own_refusal_for_a_journal_it_cannot_replay fails. With a change-transaction.json that does not parse, the dry run says an interrupted write is 'waiting to be recovered', where change approve refuses with 'invalid transaction journal at ...'. Fixed in e24bdbf9; not a new bug."
result: failed
evidence_kind: product-test
revision: 003a14e6351565712fc5cc59ddd24c8d11fbe6bc
content: 6034874e4a9003f19918a5f8789a31b9f191e2b06aa322afdcfdd65239a680f7
environment: macOS 27.2 (Darwin 27.2.0), Apple M5 Ultra, local clone
fingerprint: sha256:5c15d4cd76623e137e9d8c874391ac9e076c78423fd4dd22f41f3b09df7cd69f
external: ext-d42d9d
criteria: [CHANGE-15.a]
requirements: [REQ-change-163]
contracts: [specs/change/change.spec.md]
limitations: []
observed: 2026-11-19T07:32:11Z
actor:
  - agent: claude-code
    terminal: t59
---
**Red-first, for the tester's finding on 37496bdf: at b3573eaa (the test alone) approve_dry_run::the_dry_run_gives_the_approvals_own_refusal_for_a_journal_it_cannot_replay fails. With a change-transaction.json that does not parse, the dry run says an interrupted write is 'waiting to be recovered', where change approve refuses with 'invalid transaction journal at ...'. Fixed in e24bdbf9; not a new bug.**

Recorded as failed in macOS 27.2 (Darwin 27.2.0), Apple M5 Ultra, local clone at revision 003a14e63515, observed 2026-11-19T07:32:11Z.
