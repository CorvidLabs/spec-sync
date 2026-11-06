---
change: archived-change-status-no-longer-walks-history-per-archive-or-per-workflow-v1-change-and-read-commands-take-a-timeout
artifact: requirements
---

# Requirements

- REQ-change-135: archive-introduction index from one walk (cold path).
- REQ-change-136: the workflow-v1 cutoff verdict comes from the cutoff tree and is cached by the cutoff commit (warm path).
- REQ-change-137: deadline semantics for lifecycle reads.
- REQ-cmd-change-032: `--timeout` / `SPECSYNC_TIMEOUT` on status, show, list and ship-status; exit 124; text and JSON report.
- REQ-cli-args-024: `--timeout <secs>` parsing.

Traces SPEED-1.a, SPEED-1.c and SPEED-1.d.
