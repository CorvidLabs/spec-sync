---
change: find-an-upstream-archive-by-the-id-in-its-state-record-rather-than-its-directory-name
artifact: context
---

# Context

spec-sync#907, a review follow-up from #889. `upstream_archive_of` lists the remote default's archive directory and only reads entries named exactly `YYYY-MM-DD-<id>`. `find_change_dir` identifies archives by the `id` inside `state.json`, whatever the directory is called, so a renamed or undated package is missed and its stale active copy gets ordinary status guidance instead of the upstream-archive refusal.

Reproduced on main cd8cab5e by `upstream_archive_is_found_by_its_state_record_not_its_directory_name`: the change is archived, its package renamed to `legacy-package` and published as `origin/main`; from a branch before it, `upstream_archive_of` returns `None`.

Constraint from #889's review: the lookup must not rebuild the full archive index on the status path (lifecycle spawn ceiling, #861).
