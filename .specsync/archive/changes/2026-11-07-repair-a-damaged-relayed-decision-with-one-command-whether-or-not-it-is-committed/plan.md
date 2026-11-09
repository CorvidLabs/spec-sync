---
change: repair-a-damaged-relayed-decision-with-one-command-whether-or-not-it-is-committed
artifact: plan
---

# Plan

## Approach

Give every damaged-relay refusal in an active change one recovery, `specsync change repair-relays <id>`, that works whether or not the file is committed. It puts the original relay back from `HEAD` or from the review twin, and otherwise removes only that relay, never the approval or attempt. Archived packages keep the `git checkout HEAD` recovery.

## Out of scope

- Damage that is not a relay (invalid JSON beyond naming its recovery, edited verdicts or digests): the existing refusals stand.
- A relay on reopen, send-back or correct-owner (none exists, as in #900).
- Recording a repair in any history file.

## Steps

1. Failing tests first (done at `61e5a2b0`): the review-to-ship window for both review files, the committed approval, the no-copy removal with a block kept, nothing damaged, the archived refusal and guard, and the not-JSON refusal. Each runs the named command.
2. Pass the change record into `with_relayed_read_recovery` and name `repair-relays` for an active change.
3. Add `RelayRepair`, `RelayRepairReport` and `repair_relayed_decisions`, with a unit test for matching and the clean-record guard.
4. Add the `repair-relays` subcommand, its text and JSON output, and a grammar test.
5. Update the change and cmd_change companions, the workflow and CLI guides, and the CHANGELOG.

## Risks

- A put-back that is not byte-identical would leave `git status` dirty: the typed round trip through `json_content` and a byte-equality test cover it.
- Matching by position at `HEAD` could pick the wrong record if records were reordered: a candidate must equal the record in every other field, otherwise the relay is removed and shown, never silently swapped.
- #900's REQ-change-relayed-decision-001 and Public API rows are modified here before #900 lands; a change to them there means a CHANGE-11 rebase and one re-approval.

## Constraints consulted

change.3md "Approvals are claims; relayed decisions (#633)"; REQ-change-046 (append-only attempts, no block erased); "Keep it out"; Leif's no-copy decision (remove); never weaken a gate, bound or test.
