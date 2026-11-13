---
change: the-delivery-blob-deadline-bounds-each-git-read-not-the-manifest-work-between-reads
artifact: research
---

# Research

- `ObjectBatch::read` in src/git_utils.rs already arms `BatchWatchdog` per lookup with
  `set(Some(now + deadline))` and disarms with `set(None)`; `a_stalled_batch_read_is_killed_at_its_deadline`
  covers the kill.
- The issue's real-data reopen (site CHG-0026, 609 files, 372 MiB) took 10.5 s, so the failure
  needs a larger or slower scope; the test shortens the deadline instead.
