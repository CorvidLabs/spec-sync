---
change: make-bounded-git-timeout-cleanup-tests-independent-of-child-startup-scheduling
artifact: context
---

# Context

Issue #587 reproduces in rc.15 macOS qualification run 34164714867: the 50ms deadline kills the child before it writes child.pid. The PID-file read then fails despite successful runner cleanup. Current main is 4908a238; no open PR overlaps this fix. Scope is the bounded Git runner test seam, its regression tests, and the change module spec companions.
