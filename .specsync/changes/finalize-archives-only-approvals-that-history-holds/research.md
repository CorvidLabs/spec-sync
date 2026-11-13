---
change: finalize-archives-only-approvals-that-history-holds
artifact: research
---

# Research

- Every writer of `approvals.json` appends: `append_approval`, the portable 5.0.1 pair, the
  acceptance and finalization gates, and reopen (which appends to `reopenings[]`). They rewrite
  the file with `json_content_keeping_unknown_members`, which keeps earlier records' values.
- `change repair-relays` rewrites records, but only their `relayed_by`.
- `change check --commit` commits the approvals before review; review writes `review.json` and
  `review-attempts.json`, never `approvals.json`. The integration test
  `uncommitted_specsync_evidence_alone_does_not_refuse_review` leaves only review evidence
  uncommitted, which this check does not read.
