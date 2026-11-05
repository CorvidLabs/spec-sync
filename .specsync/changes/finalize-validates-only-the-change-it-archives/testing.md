---
change: finalize-validates-only-the-change-it-archives
artifact: testing
---

# Testing

## Requirement evidence

| Requirement | Evidence |
| --- | --- |
| REQ-change-124 | `cold_finalize_git_processes_do_not_grow_with_other_archives` failed first on the #842 base (five more archives added 329 Git processes to one cold finalize, about 82 each) and now adds 15, the three reads of each new archive's immutable introducing commit. `warm_finalize_git_processes_do_not_grow_with_other_archives` failed first (five more archives added 20 Git processes to one warm finalize) and now adds none (754 and 754). `finalize_judges_other_archives_from_their_own_bytes` failed first (finalize aborted on another archive's hidden workflow-version downgrade) and passes. `strict_audit_still_refuses_a_hidden_downgrade_in_another_archive` proves `audit_project_with_archives` names the downgraded archive, with a warm cache as well, and that the default audit is unchanged. `finalize_git_processes_do_not_grow_with_other_archives` (REQ-change-121) and the full change suite, including every finalize and archive refusal test, pass unchanged. |
| REQ-cmd-change-027 | `change audit` passes its `--archives` flag to `change::audit_project_with_archives`; `strict_audit_still_refuses_a_hidden_downgrade_in_another_archive` proves the strict report names the archived change and the non-strict report equals `audit_project`'s. |
