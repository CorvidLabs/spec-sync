---
kind: evidence
id: 2026-11-12-macos-26-after-t384-s-1067-review-changes-needed-at-4c961e79
aliases: [OBS-product-test-fe53b2b5d957d92866b57094]
claim: "macOS 26, after t384's #1067 review (CHANGES NEEDED at 4c961e79): the change reopen and change reconcile-history state.json writers now go through relay_gated_record. New integration tests: reopening_an_archived_change_whose_ledger_holds_a_relay_raises_it fails with the reopen writer ungated ((2, Some(2)) vs (3, Some(2))) and passes with it gated; a_relayed_approve_that_fails_at_the_append_has_already_raised_the_change fails without approve's early raise and passes with it. Re-run passing: relayed_decision integration 31; --bin relay 25, reopen 34, history 34, reconcil 6; integration change:: 75; clippy --all-targets -D warnings clean; fmt --check passes; strict spec check 62/62; release rebuild, then fledge run relay-gate-compat passes against the real v6.2.0 binary. change.spec.md drops the hand-added Change Log row (materialization adds its own) and goes back to version 234."
result: passed
evidence_kind: product-test
method: automated
revision: 4c961e793932f291013c80a7ad224a5424b1badd
content: 76b43f05bd9d39ed48cb8c42095265df60a8e90f9c928f0daa6fb63b62f29090
environment: local
criteria: [CHANGE-13]
requirements: [REQ-change-158]
contracts: [specs/change/change.spec.md]
limitations:
  - "The full cargo test suite was not run locally (coordinator rule: full suites run in hosted CI)"
observed: 2026-11-12T03:19:02Z
---
**macOS 26, after t384's #1067 review (CHANGES NEEDED at 4c961e79): the change reopen and change reconcile-history state.json writers now go through relay_gated_record. New integration tests: reopening_an_archived_change_whose_ledger_holds_a_relay_raises_it fails with the reopen writer ungated ((2, Some(2)) vs (3, Some(2))) and passes with it gated; a_relayed_approve_that_fails_at_the_append_has_already_raised_the_change fails without approve's early raise and passes with it. Re-run passing: relayed_decision integration 31; --bin relay 25, reopen 34, history 34, reconcil 6; integration change:: 75; clippy --all-targets -D warnings clean; fmt --check passes; strict spec check 62/62; release rebuild, then fledge run relay-gate-compat passes against the real v6.2.0 binary. change.spec.md drops the hand-added Change Log row (materialization adds its own) and goes back to version 234.**

Recorded as passed in local at revision 4c961e793932, observed 2026-11-12T03:19:02Z.
