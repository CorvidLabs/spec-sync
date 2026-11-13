---
change: finalize-archives-only-approvals-that-history-holds
artifact: design
---

# Design

`ensure_approvals_ledger_held_by_history` runs in `accept_change_with_gate` for the finalization
gate, after the verification and scoped-review checks (a missing or stale review stays the first
refusal) and before acceptance writes anything. `change ship` finalizes through it.

1. **Committed at HEAD.** The working tree's `approvals.json` must equal `HEAD:<path>`. If it is
   absent at HEAD, the refusal says to commit it (`change check --commit` does). If it differs,
   the refusal names `git checkout HEAD -- <path>`.
2. **Append-only history.** `git log --first-parent --no-renames HEAD -- :(top,literal)<path>`,
   bounded by `MAX_TRUSTED_HISTORY_COMMITS`, lists every commit that touched the file. Oldest
   first, each committed version must keep every `approvals[]` and `reopenings[]` record of the
   version before it, at the same position, equal as JSON apart from `relayed_by`. A commit
   that deleted the file, as an archive move does, is skipped, so a reopened change is compared
   with its ledger before the archive. The refusal names the rewriting commit, the earlier
   commit and the record (`approvals[0]`).

`relayed_by` is left out of the comparison because `change repair-relays` may put a relay back or
remove it, and no gate reads a relay (REQ-change-relay-repair-001). First-parent history follows
the branch being finalized, not commits merged in from elsewhere.

Out of the check: a finalize resumed after acceptance (its approvals were just written by
finalize itself), and legacy `change archive`, which runs right after `change accept` writes
approvals.
