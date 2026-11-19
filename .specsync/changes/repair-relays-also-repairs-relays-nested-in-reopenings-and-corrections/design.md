---
change: repair-relays-also-repairs-relays-nested-in-reopenings-and-corrections
artifact: design
---

# Design

Each ledger file now lists record locations instead of one array key: `approvals.json` holds
`approvals[i]` and `reopenings[i].superseded_approval`; `corrections.json` holds
`corrections[i].superseded_definition_approval` and `corrections[i].superseded_closing_approval`;
`review-attempts.json` holds `reviews[i]`; `review.json` is one record. Every location uses the
same damage test (`relay_is_damaged` with the record's decider key: `actor` for approvals,
`reviewer` for reviews), the same `HEAD` copy at the same location, and the same removal. Only the
review files have twins. `RelayRepair` gains an optional `within` naming the nested record, so
`record` stays the 1-based position in the outer array, and the printer names the place from
`file`, `record` and `within`. Every ledger, including `corrections.json`, must read after the
repair or nothing is written, as today.
