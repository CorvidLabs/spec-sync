---
change: CHG-0092-complete-buttery-ship-status-tip-class-and-ship-preflight-for-agents
artifact: requirements
---

# requirements

Buttery ship v2 for SpecSync 6 agents.

`change ship-status` classifies the current HEAD tip (product / review_only /
archive_only / other) from git path shapes, reports verification tip health,
review presence, local trust guidance (no GitHub API required), and ordered ship
stages with SHAs when available.

`change ship [id]` runs the same preflight and, when ready, finalizes the change
in one step. Without readiness it fails with blockers and the next stage.

Verifying `next_action` points agents at `ship-status` / `ship` and warns not
to merge before finalize.

Sandbox: `drills/027-ship-sequence.sh` must PASS against the release binary.
