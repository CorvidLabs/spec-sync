---
change: stop-waiting-on-a-descendant-pipe-after-the-git-child-exits
artifact: design
---

# Design

No user-interface change. The reader threads poll the child's pipe and a wake pipe. When the direct child is reaped, one byte on the wake pipe releases every reader. They keep bytes already buffered and do not wait for the write end to close.
