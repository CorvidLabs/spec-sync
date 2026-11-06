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

After the Codex review of #901 (P1 quoting, P2 outside-package restore), test-first on db2e3ab4:

- `corrupt_archive_restore_quotes_every_interpolated_word` failed on db2e3ab4 (`sh` could not parse the unquoted command for ID `safe;touch PWNED $(touch PWNED2) it's`) and passes: every runnable command in each step round-trips through `sh` to the exact argv, and nothing runs in the sandbox.
- `corrupt_archive_restore_names_the_package_only_when_the_package_changed` failed on db2e3ab4 (an intact package got the no-op archiving-commit restore) and passes: a real committed archive yields the status-JSON pointer when intact, the HEAD restore after an uncommitted edit, and the archiving-commit restore after the edit is committed.
- The integration test now runs the printed commands verbatim through a shell, with the quoted words.

Earlier results:


- Red first: `a_corrupt_archive_is_not_safe_to_leave_and_names_the_restore_that_recovers_it` with the source at main ad50bdef failed exactly on the defect: JSON `handoff.readiness` `safe` beside `terminal_evidence` `corrupt-history` (the authenticated checks before the tamper passed).
- Green: with the fix it passes, covering the uncommitted HEAD restore run verbatim, the committed tamper restored from the archiving commit, text `Next:`/`Handoff:`, and `ship-status`.
- Unit: `handoff_corrupt_archive_is_not_yet_and_names_the_restore` (HEAD, archiving-commit and unlocated forms; freeze still first), `corrupt_archive_signal_is_only_for_an_archived_corrupt_history_record`, and every existing `handoff_*` test pass. `strict_check_reports_standalone_unprovable_archived_history` now pins the new exact next action and asserts the handoff agrees. Its first run caught a real bug: outside a work tree `git diff` runs as `--no-index` and exits 1, so the probe now confirms the repository first.
- Auditor fixture (`fixtures/rust-main-v1`) with the release binary: clean → safe; uncommitted tamper → not-yet with the HEAD restore; the named command → authenticated-history and safe; committed tamper → archiving-commit form; restore from `f49d652` committed → authenticated-history and safe; clean tree.
- Cost (release, macOS, this repository, 247 archives, authenticated #861 archive): text `change status` 0.55 s → ~10.7 s on a quiet machine; interleaved under load average ~70, main 1.17/1.30/1.82 s against 18.2/19.8/21.7 s. JSON unchanged (38.5/41.0/55.1 s against 38.7/40.7/67.4 s). Git processes for text status 24 → 207. The time is the project roster walk over the 73 legacy `CHG-*` archives that the persistent cache does not reuse; follow-up #897 (plan row S9). No test, gate, bound or timeout was changed.
