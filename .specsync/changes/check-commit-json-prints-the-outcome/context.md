---
change: check-commit-json-prints-the-outcome
artifact: context
---

# Context

Issue #839: `change check --commit --json` exited 0 with empty stdout. Re-cut
as a pure bug fix per t370: no new requirement text — the behavior is already
required by REQ-change-004 (equivalent human CLI and structured agent
workflows) and cli invariant 5 (`--json` switches all output to
machine-readable JSON). The refusal path's JSON error document on stdout is
the CLI's existing convention.
