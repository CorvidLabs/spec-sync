---
change: the-delivery-blob-deadline-bounds-each-git-read-not-the-manifest-work-between-reads
artifact: testing
---

# Testing

- `the_delivery_blob_deadline_bounds_each_read_not_the_work_between_reads`
  (src/change_tests/fast_finalize.rs): a 300 ms deadline, one read, 700 ms of simulated manifest
  work, then a second read that must succeed. Red on main: the second read fails with the
  deadline error.
- `the_watchdog_counts_only_armed_reads_and_says_when_it_fired` (src/git_utils.rs): unarmed time
  never fires the watchdog; a stalled read is killed and `expired()` says so.
- Existing delivery and stalled-batch tests stay green; fmt and clippy clean.
