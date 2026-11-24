---
change: the-upstream-archive-lookup-reads-tree-entries-as-bytes-and-knows-an-archive-head-already-holds-after-an-upstream
artifact: testing
---

# Testing

## Requirement evidence

| Requirement | Evidence |
|---|---|
| REQ-change-129 (#945) | `a_reopened_change_reviews_after_the_remote_default_renames_its_archive` in `upstream_archive.rs`, the real flow through the lifecycle functions the commands call (finalize, reopen, check, review) with a bare origin, plus its control with no rename (the tester's BLOCK on f740d0a2: red there, green after containment moved from HEAD's tree to HEAD's history); `an_upstream_rename_of_an_archive_head_already_contains_is_not_reported` and the existing `upstream_archive_is_reported_only_when_head_does_not_contain_it` |
| REQ-change-129, REQ-change-142 (#961) | `an_unrelated_non_utf8_archive_package_does_not_stop_the_upstream_lookup` (red before, green after), and the existing cannot-tell tests |
| REQ-change-139 (bounded processes) | the existing upstream-archive tests; one `log -1 --find-object` of HEAD's history runs per matching record, only when the remote default holds a match |

## How it runs

`cargo test --bin specsync upstream_archive` locally (red, then green), and hosted CI's full suite
at the archived head.
