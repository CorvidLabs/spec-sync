---
change: the-upstream-archive-lookup-reads-tree-entries-as-bytes-and-knows-an-archive-head-already-holds-after-an-upstream
artifact: testing
---

# Testing

## Requirement evidence

| Requirement | Evidence |
|---|---|
| REQ-change-129 (#945) | `an_upstream_rename_of_an_archive_head_already_contains_is_not_reported` (red before the fix, green after), and the existing `upstream_archive_is_reported_only_when_head_does_not_contain_it` |
| REQ-change-129, REQ-change-142 (#961) | `an_unrelated_non_utf8_archive_package_does_not_stop_the_upstream_lookup` (red before, green after), and the existing cannot-tell tests |
| REQ-change-139 (bounded processes) | the existing upstream-archive tests; one extra `ls-tree` of HEAD runs only when a match exists |

## How it runs

`cargo test --bin specsync upstream_archive` locally (red, then green), and hosted CI's full suite
at the archived head.
