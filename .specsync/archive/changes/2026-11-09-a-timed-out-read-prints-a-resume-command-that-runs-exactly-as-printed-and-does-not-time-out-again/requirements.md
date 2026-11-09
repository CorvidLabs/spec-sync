---
change: a-timed-out-read-prints-a-resume-command-that-runs-exactly-as-printed-and-does-not-time-out-again
artifact: requirements
---

# Requirements

- REQ-cmd-change-036: the resume command a timed-out read prints splits back, under a POSIX shell, into exactly the arguments of the original invocation without its timeout, and clears `SPECSYNC_TIMEOUT` when that variable was set.

Traces SPEED-1.
