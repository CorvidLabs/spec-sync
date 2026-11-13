---
change: the-batched-object-reader-accepts-sha-256-object-ids
artifact: tasks
---

# Tasks

- [x] Red-first unit test: reads at a 64-character commit id in a sha256 repository are answered by one batch child and equal per-file reads
- [x] Accept 64-character hex object ids in batchable_revision
- [ ] Targeted git_utils tests, fmt and clippy
