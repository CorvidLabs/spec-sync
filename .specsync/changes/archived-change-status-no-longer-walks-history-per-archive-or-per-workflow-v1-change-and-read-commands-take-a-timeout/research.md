---
change: archived-change-status-no-longer-walks-history-per-archive-or-per-workflow-v1-change-and-read-commands-take-a-timeout
artifact: research
---

# Research

Measured `change status <archived-id> --json` on this repository: 320 archives, 619 commits, macOS, load average 66-90. Wall times vary a lot with load.

| Run | Wall | Verdict |
|---|---|---|
| Cold (empty read cache, private clone) | 83-291 s | authenticated-history |
| Warm | 49-53 s | authenticated-history |

Profiles were taken with macOS `sample` on an unstripped release build. Most samples wait on Git children (`poll`, `posix_spawn`, `wait4`), not on CPU.

- **Cold.** 85% of samples are in `load_scoped_review -> archive_introduction_index -> archive_introductions_at_commit`. That is one `git log`, then one `diff-tree` for each of the 214 commits that added archive files, then reads of each package's `state.json` and `approvals.json`. One `git log --diff-filter=A --name-only -z` over the archive root returns the same added paths in 0.20 s.
- **Warm.** 89% of samples are in `terminal_evidence_summary -> list_all_changes_loaded -> validate_workflow_version_history -> validate_workflow_v2_baseline -> workflow_v1_record_exists_at_cutoff -> workflow_version_state_paths`. For each of the 73 workflow-v1 changes this runs `rev-list --full-history HEAD -- :(top,glob)<archive>/*-<id>/state.json`, at 0.23-0.56 s each, plus `show` reads at the cutoff. The verdict is only kept per read scope, so every command redoes it.
- **Test gap.** `archived_status_git_processes_do_not_grow_with_other_archives` and `raven_sized_archived_status_benchmark` build only workflow-v2 changes, so they pass. A fixture with workflow-v1 changes made before adoption fails: 97 Git processes with 2 such changes and 115 with 8, about +3 per change.
