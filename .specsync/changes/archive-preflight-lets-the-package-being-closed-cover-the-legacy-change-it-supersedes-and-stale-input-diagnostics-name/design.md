---
change: archive-preflight-lets-the-package-being-closed-cover-the-legacy-change-it-supersedes-and-stale-input-diagnostics-name
artifact: design
---

# Design

- One token, forwarded, never minted: the successor walk gains `pending: Option<&PendingArchiveClose>`; every reader passes `None` (documented on `PendingArchiveClose`), and the archive preflight passes `pending_close.as_ref()`. `is_closing` keeps the token inert for every package except the one being closed, and a post-move resume still gets none.
- The working-tree anchor is decided by the label `authenticated_accepted_transition_for` already returns (`WORKING_TREE_CLOSING_EVIDENCE_ANCHOR`), not by the token, because the same shape exists for every reader between `finalize` and the archive commit. Once the archive commit exists, history is again the sole anchor. The successor entry is read by `acceptance_entry_digest_in_tree`, the same code the detached-worktree path uses.
- Refusals become data: `RejectedSuccessor { workflow_version, reason }` in a `BTreeMap` keyed by successor ID (deterministic order), rendered as "successor `<id>` was rejected: <reason>". The three formerly OR'd checks are split into distinct reasons, and the two `Result<bool>` predicates report a decided negative and a failure to evaluate differently (#743).
- Candidates are pre-filtered by the declared obligation (`declares_succession_obligation`, now shared with `legacy_semantic_successor_tuple`). This is equivalent to the prior behaviour — authenticated tuples are one-to-one with declared obligations, and the legacy reconstruction required the declaration — and it keeps every recorded refusal about a successor that actually claimed the input.
- Remediation depends on the two workflow versions: a v1 predecessor with a refused v2 claiming successor is steered to `specsync change finalize` via `change status <successor>` and is never offered `change reopen <predecessor>`; every other combination keeps the established verify-and-accept-or-reopen wording.
