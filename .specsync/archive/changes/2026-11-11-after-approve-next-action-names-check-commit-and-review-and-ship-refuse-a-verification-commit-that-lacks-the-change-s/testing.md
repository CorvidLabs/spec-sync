---
change: after-approve-next-action-names-check-commit-and-review-and-ship-refuse-a-verification-commit-that-lacks-the-change-s
artifact: testing
---

# Testing

These cases are the #535 reproduction on a no-spec documentation change
whose affected path is modified and uncommitted. They will be written as
integration tests through the shipped binary, red first, after definition
approval. They have not been implemented in this draft.

## Requirement evidence

| Requirement | Evidence |
|---|---|
| REQ-change-156 | After approve, text and JSON `change status` / `change show` name `specsync change check <id> --commit` and do not name a bare `change check` as the next command. The same holds in `implementing`. With one open task, the task is named first and the command that follows is `change check <id> --commit`. |
| REQ-change-157 | After a passing bare `change check`, `change review` exits non-zero, records no review, and does not bind a pass to the seed commit. `change ship` exits non-zero and does not finalize or archive. After `change check --commit` records verification against a commit that contains the affected-path work, review and ship may proceed. A tree whose only uncommitted files are under `.specsync/` is not refused by this rule. |
| REQ-change-132 | The open-task status case still names the task first; only the following check command gains `--commit`. |

The August unrecoverable-hook claim is not tested: it was re-measured as
absent and is out of scope.
