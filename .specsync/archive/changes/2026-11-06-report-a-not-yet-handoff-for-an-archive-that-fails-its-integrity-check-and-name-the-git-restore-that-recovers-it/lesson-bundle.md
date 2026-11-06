# Lesson bundle: report-a-not-yet-handoff-for-an-archive-that-fails-its-integrity-check-and-name-the-git-restore-that-recovers-it

Material for folding this change's lessons into each affected module's lesson file.
Synthesise from what actually happened below; do not restate the change description.

## What this change was

- **Title**: Report a not-yet handoff for an archive that fails its integrity check and name the git restore that recovers it
- **Kind**: BugFix
- **Specs**: change, cmd_change
- **Paths**: src/change/, src/change_tests/, src/commands/change.rs, tests/integration/change.rs, CHANGELOG.md, hi/change.md
- **Acceptance**: An archived change whose terminal evidence is `corrupt-history` reports handoff readiness `not-yet`, never `safe`, in `change status --json` (`summary.handoff`) and on the text `Handoff:` line, with a reason naming the failed integrity check.
- **Acceptance**: The step before clearing, the JSON `next_action`, the text `Next:` line and `change ship-status`'s next action name the same recovery, ending with `specsync change status <id>`: `git restore --source=HEAD --staged --worktree -- <package>` when the package differs from `HEAD`; the same restore from the commit that archived it (`git log -1 --diff-filter=A --format=%H -- <package>/state.json`), then a commit, when the package matches `HEAD` but not that commit; and, when the package still matches the commit that archived it (or no package or repository can answer), `specsync change status <id> --json` to read what `terminal_evidence.reason` names and restore that from trusted Git history. No step contains a digest, and `specsync change check`, which checks nothing for an archived change, is no longer named.
- **Acceptance**: Every change ID and path interpolated into a printed command is single-quoted as one shell word, so copying the step from a repository whose change ID or archive directory carries `;`, `$`, quotes or spaces runs exactly the named command (Codex P1 on #901).
- **Acceptance**: Running the named command verbatim through a shell on a tampered, uncommitted archive, and the archiving-commit restore on committed damage, returns terminal evidence `authenticated-history` and handoff `safe` in JSON and text; `change status` writes nothing to the archive.
- **Acceptance**: An archive whose terminal evidence authenticates, including `historical-transition-unproven`, keeps handoff `safe`, the text `Next: no further action` and the JSON next action to merge the PR on GitHub if it is still open.
- **Acceptance**: Text `change status`, and the `Handoff:` line `finalize` and `ship` print, judge an archived change with the same terminal-evidence verdict JSON reports. The measured text-status cost on this repository (247 archives) is recorded in testing.md (follow-up #897), and no test, gate, bound or timeout is weakened.
- **Acceptance**: The integration test fails on main ad50bdef with readiness `safe` and passes with the fix; the quoting and outside-package tests fail on db2e3ab4 and pass with the fix.

## Evidence

- Verification commit: `30fb8ec046c37c0fb2a82b2c0d90abd7d6066954`
- Base commit: `ad50bdef3db58a715877877d2407485bcaed0f32`
- Verified by: `specsync check --spec change --spec cmd_change`

## From the change's context.md

# Context

Audit D3 (spec-sync#880, plan row S3). After the archived verification's `workspace_digest` is replaced with zeros, `change status <id> --json` correctly reports terminal evidence `corrupt-history`, yet the same summary says `handoff.readiness: safe` with "the change is archived; nothing this session knows is still needed". An agent clearing context reads that as permission to drop the session that may know what changed the archive. Chough already refuses done on the corrupt archive; restoring the exact bytes returns `authenticated-history`, so the integrity engine is right and only the guidance contradicts it.

Reproduced on main ad50bdef with the auditor's fixture (`fixtures/rust-main-v1`, commit "Archive greeting-version-1"):

- JSON: `summary.handoff.readiness` is `safe` beside `summary.terminal_evidence.validity` `corrupt-history`.
- Text: `Next: no further action` and `Handoff: safe`. Text mode never asks about archive integrity: `handoff_summary` skips the terminal-evidence walk for an archived record by design, because it was "`safe` regardless".
- The existing JSON next action ends "then run `specsync change check`", which for an archived change prints "Nothing to check" and confirms nothing.

Recovery, checked on the same fixture with the current integrity engine: an uncommitted tamper is undone by `git restore --source=HEAD --staged --worktree -- <package>`; a committed tamper by the same restore from the commit that archived the package, and the archive stays `authenticated-history` after that restore is committed. A shallow clone is a different failure (status refuses outright with a baseline-cutoff error) and is out of scope.

Cost, measured on this repository (247 archives, release build, three runs each, text `change status` on the authenticated #861 archive): 0.55 s today; about 10.7 s when text uses the same terminal-evidence verdict as JSON; about 3.7 s for an own-package-only check. JSON status there already takes about 20 s. The extra cost of the JSON-equal verdict is the project roster walk inside `terminal_evidence_summary` (73 `rev-list` plus 73 `log` over the legacy `CHG-*` archives) that the persistent read cache does not absorb on repeated runs; that is a performance follow-up, not part of this change.

Out of scope: speeding up the roster walk; shallow-clone behaviour; any change to the integrity engine itself.

## From the change's design.md

# Design

One signal, gathered where terminal evidence already is, feeds every projection.

- `HandoffSignals` gains `corrupt_archive: Option<CorruptArchive>`: `Some` only for an archived record whose terminal evidence is `corrupt-history`. `CorruptArchive` holds the package path relative to the project root (`None` when the change does not resolve to exactly one archive directory) and whether the package's tracked files differ from `HEAD` (`git diff --quiet HEAD -- <package>` exit 1). No digest, so the recovery built from it may reach a text line.
- `classify_handoff`: `Archived` with `corrupt_archive` is `NotYet`, reason "the archive fails its integrity check, so a fresh session cannot trust it, and only this session may know what changed it", one step from `corrupt_archive_recovery`. Every other archived record stays `Safe` with today's reason.
- `CorruptArchive` is an enum (revised after the Codex review of #901): `DiffersFromHead { package }` when `git diff --quiet HEAD -- <package>` exits 1; `DamageCommitted { package }` when the package matches `HEAD` but differs from the commit that archived it (`git log -1 --diff-filter=A --format=%H -- <package>/state.json`); `OutsidePackage` when the package still matches that commit, cannot be located, or there is no repository. The repository is confirmed first, because outside a work tree `git diff` silently runs as `--no-index` and exits 1.
- `corrupt_archive_recovery(id, corrupt)`: the HEAD restore, the archiving-commit restore plus a commit, or `specsync change status <id> --json` to read what `terminal_evidence.reason` names; each then `specsync change status <id>`. Every interpolated ID and path goes through a private `shell_quote` (single quotes, `'` closed-escaped-reopened, the same quoting `commands::merge` uses), since `validate_change_id` allows `;`, `$` and spaces and the archive directory name comes from the filesystem.
- `summarize_change`: the archived next action becomes that same step (replacing "… then run `specsync change check`"); the authenticated archive keeps "merge the PR on GitHub if it is still open".
- `handoff_summary` (text paths: `status`, `show`, `finalize`, `ship`): for an archived record it now reads `terminal_evidence_summary` — the same verdict JSON carries — instead of assuming `safe`. Inside a read scope that verdict is memoized, so `Next:` and `Handoff:` share one walk.
- `text_mode_next_action` (and so `change ship-status`): an archived record asks the domain through `change::corrupt_archive_recovery_for_guidance`, which returns the same step or `None`; `None` keeps "no further action".

Rejected: an own-package-only check for the text paths (about 3.7 s instead of 10.7 s here). It can disagree with JSON whenever the project roster fails, which is the contradiction this change removes.

## From the change's testing.md

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

## Where these lessons go

- `specs/change/change.3md`
- `specs/cmd_change/cmd_change.3md`
