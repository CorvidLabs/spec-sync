---
change: keep-check-fix-export-rows-in-one-contiguous-table-for-issue-768
artifact: context
---

# Context

## Evidence and scope
Issue #768 remains open and unassigned. Its latest maintainer comment says #770 fixed only case 3; cases 1 and 2 remain. At selection, the only open PR was excluded #800. Fresh main was 9003fad7ec4dea3938179d076248b48f29b6ed48. No generate binding files are in scope.

Read cmd_check.spec.md and all companions, including authoritative cmd_check.3md. Preserve deterministic local fixes, code examples, and export-kind routing.

## Narrowed scope after reproduction
Case 1 is this patch. Case 2 remains open: the insertion-only fix still duplicates an existing symbol because parser fence blanking hides its documentation. No parser files are changed here. The initial two-test red log records discovery; the final regression isolates case 1 with nine layout/header combinations.

## Ownership recheck
Main remained 9003fad on 2026-10-11. Open PRs are now #800 and #814; #814's file list has no overlap with this patch. Issue #768 comments and assignment are unchanged.

## Human scope authorization
The user was asked: "For #768, the scope is just the wrong-table insertion bug, its regression tests, and matching spec/docs. The fenced-code parsing bug stays separate. Approve that scope and a draft PR once validation is sorted?"
The user (Sentinel_fa7dbbab35548191ad234f735456fa69) answered: "yes alweasy test locally first and make sure you are 95% confident on your fix or change and that it doesn't cause any regressions".
This authorizes the bounded case-1 scope and draft publication after strong local validation. It is not implementation review, acceptance, merge permission, or a literal measured probability. No case-2 parser changes are included.
