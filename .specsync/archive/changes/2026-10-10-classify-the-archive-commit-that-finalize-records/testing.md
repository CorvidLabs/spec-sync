---
change: classify-the-archive-commit-that-finalize-records
artifact: testing
---

# Testing

`cargo test --test integration -- eight_workflow_v2_changes_finalize` passed. Each of the eight finalizations classified `archive_only=true`, and the archive commit's parent was the implementation commit.
