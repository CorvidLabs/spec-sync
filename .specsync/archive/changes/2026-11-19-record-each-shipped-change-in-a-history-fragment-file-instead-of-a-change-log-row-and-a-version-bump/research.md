---
change: record-each-shipped-change-in-a-history-fragment-file-instead-of-a-change-log-row-and-a-version-bump
artifact: research
---

# Research

Done on 2026-11-19 with a release build of main at 65864de8 (`specsync 6.3.0`), in a scratch Git repository with a workflow-v2 policy, one `auth` spec at `version: 1` with a one-row Change Log, and no `.gitattributes` (no merge driver). Two branches cut from `main` each ran `change new`, `answer`, `approve` and `change check --commit` for one change to `auth`: branch `a` added REQ-auth-002, branch `b` modified the `Invariants` section.

| Step | Result |
|---|---|
| `git merge --no-edit b` on `a` | exit 1, `CONFLICT (content): Merge conflict in specs/auth/auth.spec.md` |
| The conflict | the two Change Log rows, appended after the same line. The two `version: 1` to `2` bumps were identical and merged; once one branch is behind the other they differ and conflict too |
| `requirements.md`, the `Invariants` section | merged cleanly |

The code on main:

- `prepare_module_application` (`src/change/collect_requirement_ids.rs`) bumps `version:` and appends the row when `changelog_records_change` finds no `<id>:` in the Change Log; that row is the convergence evidence from #741.
- `send_back.rs` reads the same row to list applied modules, to find the pre-change commit (`git log -S '<id>:'`) and to name a dropped module's row and bump for removal.
- `lifecycle_commit_scope`, `record_covers_project_path` and the acceptance-owner resolver enumerate the spec and its companions as the change's canonical files.
- The validation snapshot digest (`hash_cache.rs`) frames the spec, its companions, its `files:` and global inputs, so a new file beside the spec would be replayed over without a digest change.
- Released 6.3.0 `check` reads only `*.spec.md` and companions, so it ignores a `history/` directory. A 6.3.0 `change check` over a change this build materialized finds no row, so it would add a row and a bump as well; REQ-validator-048 reports such a double record.
