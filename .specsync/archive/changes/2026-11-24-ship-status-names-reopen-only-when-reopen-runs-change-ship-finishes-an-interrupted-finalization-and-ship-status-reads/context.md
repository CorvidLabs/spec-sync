---
change: ship-status-names-reopen-only-when-reopen-runs-change-ship-finishes-an-interrupted-finalization-and-ship-status-reads
artifact: context
---

# Context

- **#922** (the issue body and Leif's 2026-11-06 comment): ship-status's product-tip stage names `change reopen` for an accepted or archived change whenever whole-project verification is not current. `change reopen` judges only the change's own accepted evidence, so after a commit that changes an unrelated file it refuses ("delivery inputs are current"), and with an unreadable `verification.json` it fails ("invalid verification evidence"). For an archived change with stale evidence, `ship_next` says `change check <id> --commit`, which refuses an archived change.
- **#933** (Codex P2 on #903): REQ-cmd-change-031's open-task refusal names `specsync change ship <id>`, and text status tells an accepted workflow-v2 change to run `change ship` or `change finalize`; ship readiness accepts only a verifying change, so after a finalization that stopped between acceptance and the archive move, the named command refuses.
- **#938** (Codex P2 on #934): ship-status evaluates the sequence-ledger freeze twice for each report.
- **#1038 follow-up**: ss-lead, 2026-11-19, the assertion for the `historical-transition-unproven` arm goes into this change.
- The spike on 4d439b39 reproduced all three #922 cases; `validate_accepted_inputs_recursive` agreed with `change reopen` in each. The unapproved 11-13 branch `fix/922-product-stage-reopen-only-when-stale` mapped terminal evidence to a state, where an archived change never reads stale; its fix is redone and its two failing tests are kept.
- Leif's decision, 2026-11-18, relayed by hc t4c through ss-lead (hc t58): fix #938 with the ship-status batch, together with #922 and #933, as a new change with its own definition. #933 takes the issue's option 1.
