---
change: finalize-validates-only-the-change-it-archives
artifact: testing
---

# Testing

## Requirement evidence

| Requirement | Evidence |
| --- | --- |
| REQ-change-124 | `cold_finalize_git_processes_do_not_grow_with_other_archives` failed first on the #842 base (four more archives added 329 Git processes to one cold finalize: 794 with one other archive, 1,123 with five) and passes with the structural roster. `finalize_judges_other_archives_from_their_own_bytes` failed first (finalize aborted on another archive's hidden workflow-version downgrade) and passes. `strict_audit_still_refuses_a_hidden_downgrade_in_another_archive` proves `change audit --strict` names the downgraded archive, with and without a warm read cache, and that the default audit is unchanged. `finalize_git_processes_do_not_grow_with_other_archives` (REQ-change-121) and the full change suite, including every finalize and archive refusal test, pass unchanged. |
