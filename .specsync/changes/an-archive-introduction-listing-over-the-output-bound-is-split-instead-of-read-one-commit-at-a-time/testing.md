---
change: an-archive-introduction-listing-over-the-output-bound-is-split-instead-of-read-one-commit-at-a-time
artifact: testing
---

# Testing

| Requirement | Evidence |
|---|---|
| REQ-change-135 | `change::tests::an_over_bound_archive_introduction_listing_is_split_not_read_per_commit` (red: `None`; green after), `cold_archive_introduction_index_does_not_grow_with_archive_introductions` and `one_walk_archive_introductions_equal_per_commit_introductions` |

Locally: `cargo test --bin specsync archive_introduction`; hosted CI runs the full suite at the archived head.
