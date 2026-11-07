---
change: a-warm-status-of-an-archived-change-does-not-digest-the-whole-working-tree-to-report-its-review-currency
artifact: context
---

# Context

After #979 fixed the cold path, the auditor's Raven dogfood (`_audits/dogfood-raven-latency-2026-11-06.md`) still put spec-sync's warm `change status` at 3.6 s or more on a loaded host, against SPEED-1.a's 2 s. The orc asked for a warm-path profile and fix, with REQ-change-147. SPEED-1.a covers warm reads, so there is no new HI.

On Raven (150 archives, 5,629 tracked files) a warm `change status <archived-id> --json` spawns 183 Git processes, about 8 s at load average 58. A symbolized macOS `sample` puts 62% of samples under `summarize_change_with_effective -> scoped_review_currency -> project_input_digest`. The summary computes `scoped_review_current` for every state, archived included. That takes the whole-tree project-input digest, two passes of 13 path chunks with 5 Git commands each: 130 of the 183 processes. The other 33% is `terminal_evidence_summary` answering cached archives.
