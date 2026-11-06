## ADDED

### REQUIREMENT REQ-change-135

The archive-introduction index SHALL be built from one `git log` walk of the archive root that names the paths each commit added, and SHALL read package bytes through the per-command object batch reader, so the Git processes a cold archived `change status` spawns SHALL NOT grow with the number of commits that introduced archives. The index SHALL hold the same introductions, and every refusal it backs SHALL be unchanged (SPEED-1.a).

Acceptance Criteria
- A cold archived status spawns fewer than 3 more Git processes after 4 more archive-introducing commits.
- The index built from one walk equals the per-commit index on the same history, including merge commits and packages added in more than one commit.
- The deterministic history bound still refuses a history over it.

### REQUIREMENT REQ-change-136

Whether a workflow-v1 change existed at the trusted pre-v2 cutoff SHALL be decided from the cutoff commit's own tree, read once per command for all workflow-v1 changes, and the verdict SHALL be kept in the persistent read cache keyed by the cutoff commit, which never changes. A warm archived `change status` SHALL NOT spawn more Git processes as the project holds more workflow-v1 changes (SPEED-1.a, SPEED-1.c).

Acceptance Criteria
- `warm_archived_status_git_processes_do_not_grow_with_legacy_changes`: six more workflow-v1 changes add fewer than 3 Git processes.
- A workflow-v1 change absent at the cutoff, added after it, or present at it only under another id is still refused with the existing message.
- A workflow-v1 change archived or moved after the cutoff is still accepted.
- A hidden workflow downgrade is still refused, cached or not.

### REQUIREMENT REQ-change-137

A read-only lifecycle read given a deadline SHALL stop at the next phase boundary once the deadline passes, SHALL write nothing to the change, SHALL flush every verdict it completed to the persistent read cache, and SHALL report the phase it was in, its progress within that phase when known, and the exact command that resumes it.

Acceptance Criteria
- A read stopped by its deadline leaves every change file byte-identical.
- Verdicts completed before the deadline are in the read cache afterwards, so the next run does not redo them.
- The report names the phase and progress, for example `archive-introduction index` and `140/214`.
