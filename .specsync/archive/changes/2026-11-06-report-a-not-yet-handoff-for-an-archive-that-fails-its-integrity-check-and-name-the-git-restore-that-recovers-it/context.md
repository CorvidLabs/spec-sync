---
change: report-a-not-yet-handoff-for-an-archive-that-fails-its-integrity-check-and-name-the-git-restore-that-recovers-it
artifact: context
---

# Context

Audit D3 (spec-sync#880, plan row S3). After the archived verification's `workspace_digest` is replaced with zeros, `change status <id> --json` correctly reports terminal evidence `corrupt-history`, yet the same summary says `handoff.readiness: safe` with "the change is archived; nothing this session knows is still needed". An agent clearing context reads that as permission to drop the session that may know what changed the archive. Chough already refuses done on the corrupt archive; restoring the exact bytes returns `authenticated-history`, so the integrity engine is right and only the guidance contradicts it.

Reproduced on main ad50bdef with the auditor's fixture (`fixtures/rust-main-v1`, commit "Archive greeting-version-1"):

- JSON: `summary.handoff.readiness` is `safe` beside `summary.terminal_evidence.validity` `corrupt-history`.
- Text: `Next: no further action` and `Handoff: safe`. Text mode never asks about archive integrity: `handoff_summary` skips the terminal-evidence walk for an archived record by design, because it was "`safe` regardless".
- The existing JSON next action ends "then run `specsync change check`", which for an archived change prints "Nothing to check" and confirms nothing.

Recovery, checked on the same fixture with the current integrity engine: an uncommitted tamper is undone by `git restore --source=HEAD --staged --worktree -- <package>`; a committed tamper by the same restore from the commit that archived the package, and the archive stays `authenticated-history` after that restore is committed. A shallow clone is a different failure (status refuses outright with a baseline-cutoff error) and is out of scope.

Cost, measured on this repository (247 archives, release build, three runs each, text `change status` on the authenticated #861 archive): 0.55 s today; about 10.7 s when text uses the same terminal-evidence verdict as JSON; about 3.7 s for an own-package-only check. JSON status there already takes about 20 s. The extra cost of the JSON-equal verdict is the project roster walk inside `terminal_evidence_summary` (73 `rev-list` plus 73 `log` over the legacy `CHG-*` archives) that the persistent read cache does not absorb on repeated runs; that is a performance follow-up, not part of this change.

Out of scope: speeding up the roster walk; shallow-clone behaviour; any change to the integrity engine itself.
