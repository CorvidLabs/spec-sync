---
change: name-clean-git-files-by-object-id-so-change-check-accepts-an-lfs-pointer
artifact: testing
---

# Testing

## Requirement evidence

| Requirement | Evidence |
|-------------|----------|
| REQ-change-103 | `git_lfs_clean_file_is_named_by_its_object_id` names a clean `filter=lfs` file by its Git object id, shows that a dirty copy changes the workspace digest, and leaves other content filters fail-closed. Scoped acceptance tests still hash covered path bytes. |

`git_lfs_clean_file_is_named_by_its_object_id` builds a repository whose clean and smudge filters are `cat`, commits a pointer, and checks that a clean `filter=lfs` file is named by its object id. It then rewrites the worktree file and checks that the workspace digest changes.

`custom_content_attributes_fail_before_index_substitution` and `fsmonitor_valid_and_ident_conversion_fail_closed` keep every other content filter fail-closed. Scoped acceptance tests still compare payload digests to file bytes. The payload bound stays 256 MiB for bytes that are read.
