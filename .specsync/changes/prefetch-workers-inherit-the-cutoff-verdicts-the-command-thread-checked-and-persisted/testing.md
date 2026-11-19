---
change: prefetch-workers-inherit-the-cutoff-verdicts-the-command-thread-checked-and-persisted
artifact: testing
---

# Testing

| Requirement | Evidence |
|---|---|
| REQ-change-136 | `change::tests::prefetch_workers_leave_every_cutoff_verdict_in_the_read_cache` (red: no verdict kept; green after), and the existing cutoff and worker tests |

Locally: `cargo test --bin specsync cutoff` and `prefetch`; hosted CI runs the full suite at the archived head.
