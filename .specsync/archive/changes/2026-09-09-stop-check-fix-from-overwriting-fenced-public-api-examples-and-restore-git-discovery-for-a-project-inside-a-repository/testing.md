---
change: stop-check-fix-from-overwriting-fenced-public-api-examples-and-restore-git-discovery-for-a-project-inside-a-repository
artifact: testing
---

# Testing

| ID | Evidence |
|----|----------|
| REQ-cmd-check-016 | `fix_preserves_fenced_example_when_renaming_a_near_miss_header` (plus existing `fix_ignores_fenced_exported_heading_before_the_table`, `fix_near_miss_handles_levenshtein_typos`) |
| REQ-git-utils-005 | `git_inherited_env_excludes_secrets_and_git_overrides`, `is_git_repo_detects_project_inside_repository_subdirectory` |
| REQ-cmd-new-001 | existing `new_auto_detects_single_source_file`; spec prose sync only |
