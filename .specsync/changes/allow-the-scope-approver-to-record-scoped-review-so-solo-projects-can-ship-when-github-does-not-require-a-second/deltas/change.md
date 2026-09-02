## MODIFIED

### REQUIREMENT REQ-change-046

Agent-authored changes SHALL receive one scoped review of implementation evidence before finalization. The reviewer claim MAY be the same actor as the definition approver. SpecSync SHALL NOT invent a second-person requirement beyond GitHub's merge rules.

Acceptance Criteria

- Review input contains only the change package, implementation diff, canonical semantic delta, and targeted evidence.
- The result binds the implementation parent commit, those input digests, an explicit pass/block verdict, a stable reviewer claim, and the exact required GitHub Actions check whose authenticated result is proven again by finalization.
- The reviewer claim MAY equal the definition approver (comparison is still case-insensitive for identity, not for refusal).
- Every review attempt is append-only; `review.json` is only the latest projection and cannot erase a prior blocking result.
- Native review recording and finalization run the same every-parent verification-freshness validator as project checking.
- Every intervening commit is inspected against every parent; any implementation change, including change-then-revert history, stales the review, while the metadata/archive-only finalization commit does not rerun or stale it.
- Scoped-review currency remains the three-valued answer `current` / `stale` / `unavailable` as already specified.
- Content is decided before history: a review whose recorded contract, execution, or workspace digest no longer matches the tree is `stale` for that reason.
- Finalization fails when a required scoped review is missing or blocking.
- Status states when review is needed and directs the user to open or update the PR so the configured scoped-review check runs.
- SpecSync does not refuse a ship solely because the reviewer is the definition approver.
