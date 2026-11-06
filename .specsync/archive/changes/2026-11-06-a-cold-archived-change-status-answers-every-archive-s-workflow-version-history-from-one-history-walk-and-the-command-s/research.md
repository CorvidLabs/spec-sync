---
change: a-cold-archived-change-status-answers-every-archive-s-workflow-version-history-from-one-history-walk-and-the-command-s
artifact: research
---

# Research

All reads are `change status <archived-id> --json` with no read cache, on #918's release build.

| Where | Wall | Git processes |
|---|---|---|
| This repository (320 archives), load average about 7 | 23.1 s | 5,608 |
| Raven-sized benchmark (124 archives, 24 workflow-v1), load average 6-8 | 34.1 s | 3,899 |
| Guard fixture, 2 then 6 archives | | 99, then 234 |

A macOS `sample` profile of 30 s of a cold read puts 90% of samples under `terminal_evidence_summary -> list_all_changes_loaded -> prefetch_archived_workflow_histories -> validate_workflow_version_history_uncached`: each archive's own workflow-version history check, run once per archive by up to 8 prefetch workers.

The Git processes on this repository, by shape:

| Process | Count | Asked by |
|---|---|---|
| `show <commit>:<state path> --` | 2,393 | `workflow_version_at_commit`, every state path at every commit and parent |
| `ls-tree --full-tree -z --name-only <commit> -- <path>` | 1,626 | `commit_tree_lists_path`, after a failed `show` |
| `rev-list --parents -n 1` | 481 | the parents of each history commit |
| `rev-list --reverse --full-history` | 333 | each archive's state-path history |
| `rev-list --full-history --max-count` and `log --full-history --name-only` | 323 + 323 | `workflow_version_state_paths` |

Workers have a seeded fact memo but no batch reader (REQ-git-utils-011), so every blob read and path check is its own process.

One walk answers the history questions for all archives at once. On this repository `git log --full-history -m --name-only -z --format=%x01%H%x20%P HEAD -- ':(top,glob).specsync/changes/*/state.json' ':(top,glob).specsync/archive/*/state.json'` lists 170 commits in 0.05 s. For a merge it prints the commit once per parent whose diff touches the pathspec, but a block does not say which parent it is against. So the per-parent listing must come from a form that names the parent: `diff-tree --stdin` fed `<commit> <parent>` pairs, or the parent order checked by a test. That is settled at build time by the equivalence tests, not assumed.
