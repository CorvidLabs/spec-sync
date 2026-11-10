## ADDED

### REQUIREMENT REQ-change-relay-compare-001

Every comparison that decides whether an approval or correction history is current, intact or a lawful successor SHALL read a relayed decision as absent, as the review digest and both correction digests already do: the approval-succession check of a reopened package, the definition digest (and so the execution digest) over the correction ledger, and the trusted correction-history comparison SHALL clear `relayed_by` from top-level and embedded approvals before comparing or hashing. Every other field SHALL still be compared exactly.

#### Acceptance Criteria

- A reopened package whose `approvals.json` an older 6.x binary rewrote without `relayed_by`, on its approvals and on a reopen event's superseded approval, is a lawful successor; any other edit to an earlier approval, or a superseded approval that is not the one the package closed on, is not.
- The definition digest is the same whether a correction's embedded approvals carry a relay, carry an edited one or carry none; any other edit to a correction changes it.
- A correction ledger an older 6.x binary rewrote without its embedded relays matches its trusted history; any other edit to a committed correction is still divergence.
