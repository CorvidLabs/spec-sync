---
change: prefetch-workers-share-one-copy-of-the-command-s-seeded-repository-facts
artifact: testing
---

# Testing

| Requirement | Evidence |
|---|---|
| REQ-git-utils-011 | `git_utils::tests::a_repository_fact_seed_shares_one_copy_of_its_facts_across_workers` (red: does not compile against main's owned seed; green after), and `cold_archived_status_prefetch_workers_reuse_repository_facts` |

Locally: `cargo test --bin specsync git_utils::` and `prefetch`; hosted CI runs the full suite at the archived head.
