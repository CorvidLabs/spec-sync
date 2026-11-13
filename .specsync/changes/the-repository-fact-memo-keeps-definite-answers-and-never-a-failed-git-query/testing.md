---
change: the-repository-fact-memo-keeps-definite-answers-and-never-a-failed-git-query
artifact: testing
---

# Testing

| Requirement | Evidence |
|---|---|
| REQ-git-utils-009 | `git_utils::tests::a_failed_fact_is_not_memoized_but_a_definite_absence_is` (red: "a failed query was memoized"; green after), and the existing memo tests |
| REQ-change-127 | `eight_workflow_v2_changes_finalize_in_their_originating_prs_without_duplicate_product_ci`, which asserts the spawn ceiling (run under the shared lock) |

Hosted CI runs the full suite at the archived head.
