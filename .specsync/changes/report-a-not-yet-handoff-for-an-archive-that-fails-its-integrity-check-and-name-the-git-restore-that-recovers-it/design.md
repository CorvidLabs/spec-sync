---
change: report-a-not-yet-handoff-for-an-archive-that-fails-its-integrity-check-and-name-the-git-restore-that-recovers-it
artifact: design
---

# Design

One signal, gathered where terminal evidence already is, feeds every projection.

- `HandoffSignals` gains `corrupt_archive: Option<CorruptArchive>`: `Some` only for an archived record whose terminal evidence is `corrupt-history`. `CorruptArchive` holds the package path relative to the project root (`None` when the change does not resolve to exactly one archive directory) and whether the package's tracked files differ from `HEAD` (`git diff --quiet HEAD -- <package>` exit 1). No digest, so the recovery built from it may reach a text line.
- `classify_handoff`: `Archived` with `corrupt_archive` is `NotYet`, reason "the archive fails its integrity check, so a fresh session cannot trust it, and only this session may know what changed it", one step from `corrupt_archive_recovery`. Every other archived record stays `Safe` with today's reason.
- `corrupt_archive_recovery(id, corrupt)`: package differs from `HEAD` → `git restore --source=HEAD --staged --worktree -- <package>` then `specsync change status <id>`; package matches `HEAD` (damage committed) → the same restore from the archiving commit, named by `git log -1 --diff-filter=A --format=%H -- <package>/state.json`, commit it, then `specsync change status <id>`; package unknown → restore under `.specsync/archive/changes/` from trusted Git history, then `specsync change status <id>`.
- `summarize_change`: the archived next action becomes that same step (replacing "… then run `specsync change check`"); the authenticated archive keeps "merge the PR on GitHub if it is still open".
- `handoff_summary` (text paths: `status`, `show`, `finalize`, `ship`): for an archived record it now reads `terminal_evidence_summary` — the same verdict JSON carries — instead of assuming `safe`. Inside a read scope that verdict is memoized, so `Next:` and `Handoff:` share one walk.
- `text_mode_next_action` (and so `change ship-status`): an archived record asks the domain through `change::corrupt_archive_recovery_for_guidance`, which returns the same step or `None`; `None` keeps "no further action".

Rejected: an own-package-only check for the text paths (about 3.7 s instead of 10.7 s here). It can disagree with JSON whenever the project roster fails, which is the contradiction this change removes.
