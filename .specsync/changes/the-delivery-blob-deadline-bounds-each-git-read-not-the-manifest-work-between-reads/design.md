---
change: the-delivery-blob-deadline-bounds-each-git-read-not-the-manifest-work-between-reads
artifact: design
---

# Design

Reuse `git_utils::BatchWatchdog`, the per-read watchdog `ObjectBatch` already uses, instead of a
second watchdog: it becomes `pub(crate)` and records whether it fired (`expired()`), which the
delivery stream needs to word its deadline error. `DeliveryBlobStream::read` arms the deadline
just before it writes the object id and disarms it after the blob and its terminator are read,
exactly as `ObjectBatch::read` does. Between reads the watchdog waits unarmed.

- change module: `DeliveryBlobStream` holds a `BatchWatchdog` and its deadline;
  `open_with_deadline` lets tests shorten the deadline. Drop closes stdin and reaps the child;
  the watchdog's own Drop stops its thread.
- git_utils module: `BatchWatchdog` is crate-visible, with `expired()`. `ObjectBatch` is
  unchanged.

Rejected: reading every blob's digest first and resolving owners afterwards. It reorders the
manifest builder and holds more state, for the same result.
