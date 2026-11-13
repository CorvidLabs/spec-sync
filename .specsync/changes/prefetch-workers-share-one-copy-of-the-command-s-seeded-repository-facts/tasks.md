---
change: prefetch-workers-share-one-copy-of-the-command-s-seeded-repository-facts
artifact: tasks
---

# Tasks

- [x] Red-first unit test: per-worker seed clones share one copy, and a worker answers from it (does not compile against main)
- [ ] Share the seed's facts behind an Arc; read them as a frame's inherited facts
- [ ] git_utils and prefetch tests, fmt and clippy
