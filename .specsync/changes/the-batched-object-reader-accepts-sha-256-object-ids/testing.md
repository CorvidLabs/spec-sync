---
change: the-batched-object-reader-accepts-sha-256-object-ids
artifact: testing
---

# Testing

## Requirement evidence

| Requirement | Evidence |
|---|---|
| REQ-git-utils-010 | `git_utils::tests::batched_reads_accept_sha256_object_ids` (red before the fix, green after), plus the existing `batched_reads_equal_per_file_reads_from_one_child` |

## How it runs

`change check` runs only the in-process spec-code sync. The unit tests run locally with
`cargo test --bin specsync git_utils::tests::batched`, and in hosted CI's full suite at the
archived head.
