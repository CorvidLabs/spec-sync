---
change: CHG-0001-bootstrap-and-ship-the-verified-specsync-5-0-full-sdd-lifecycle
artifact: design
---

# Design

Use `.specsync/sdd.json` for policy and `.specsync/changes/CHG-NNNN-slug/` for Git-versioned state. Keep requirements and semantic deltas human-readable, while state, approvals, and verification evidence use deterministic JSON. Validate the effective canonical contract plus approved deltas against source exports.
