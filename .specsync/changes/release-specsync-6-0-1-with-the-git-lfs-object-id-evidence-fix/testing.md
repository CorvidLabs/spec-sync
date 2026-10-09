---
change: release-specsync-6-0-1-with-the-git-lfs-object-id-evidence-fix
artifact: testing
---

# Testing

## Requirement evidence

| ID | Evidence |
| --- | --- |
| REQ-github-002 | `python3 -S .github/scripts/validate-release-version.py` checks the Action default, consumer pin, Trust pin, README and site Action examples, the inputs-table default, and the changelog link. |

The LFS behavior this release publishes is already covered by `git_lfs_clean_file_is_named_by_its_object_id`, `clean_object_id_does_not_match_a_file_of_those_bytes`, `filter_lfs_without_the_git_lfs_driver_fails_closed`, and `filter_lfs_rejects_a_blob_that_is_not_a_pointer`.
