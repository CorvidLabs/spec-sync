---
change: report-a-not-yet-handoff-for-an-archive-that-fails-its-integrity-check-and-name-the-git-restore-that-recovers-it
artifact: testing
---

# Testing

Failing test first, through the built binary, under workflow v2.

1. Integration (`tests/integration/change.rs`): a fresh workflow-v2 project; one change approved, checked with `--commit`, reviewed and finalized; the archive committed.
   - Authenticated: JSON `terminal_evidence` `authenticated-history`, handoff `safe`; text `Next: no further action`, `Handoff: safe`.
   - Uncommitted tamper (archived `verification.json` `workspace_digest` set to zeros): JSON `corrupt-history`, handoff `not-yet`, `before_clearing[0]` and `next_action` equal and name `git restore --source=HEAD --staged --worktree -- <package>` and `specsync change status <id>`; text `Next:` names the same step and the `Handoff:` line reads `not yet` with it under `Before clearing:`; `change ship-status` names it; no 40-hex word on any of those lines; status leaves the tampered bytes as they were.
   - Run the command extracted from `before_clearing[0]` verbatim: JSON `authenticated-history` and `safe`, text `safe`, `git status` clean.
   - Committed tamper: the step names the archiving-commit form; run it with the commit the named `git log` prints, commit the restore: `authenticated-history` and `safe` again.
   This test must fail on main ad50bdef (readiness `safe`).
2. Unit (`classify_handoff`, no repository): corrupt archive → `NotYet` with the HEAD form, the archiving-commit form and the unknown-package form; an authenticated archive stays `Safe`; the corrupt step never carries a digest; the sequence freeze still outranks it.
3. Existing assertions that pinned the old next-action text ("… then run `specsync change check`") are updated to the new exact text, not loosened.
4. Cost: text `change status` on an authenticated archive in this repository, before and after, three runs each, recorded below.
5. `specsync change check <id> --commit`, `fledge lanes run verify`, `fledge lanes run pre-push`.

## Results

Pending.
