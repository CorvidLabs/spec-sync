## ADDED

### REQUIREMENT REQ-cmd-change-relay-repair-001

`change repair-relays <id>` SHALL run `repair_relayed_decisions` and print one line for each record it repaired: the file and the record's position, then where the original was put back from, or that it was removed together with the removed value. `--json` SHALL print the repair report.

#### Acceptance Criteria

- A put-back prints `<place>: put back from <copy>`, where the place is `approvals.json approval <n>`, `review-attempts.json attempt <n>` or `review.json`, and the copy is `HEAD`, `review.json` or `review-attempts.json attempt <n>`.
- A removal prints `<place>: removed`, says that no copy of the original exists, and shows the removed value with control characters escaped.
- With nothing damaged it prints `No relayed decision in <id> needs repair.` and exits 0.
- A refusal exits non-zero and names its recovery command.
- `--json` prints `{"change": <id>, "repairs": [...]}` with each repair's `file`, `record`, `action`, and `from` or `removed`.
