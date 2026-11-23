---
change: in-a-shallow-clone-name-git-fetch-unshallow-origin-in-every-lifecycle-history-refusal-and-refuse-change-new-before
artifact: testing
---

# Testing

## Requirement evidence

| Requirement | Evidence |
|---|---|
| REQ-change-173 | `tests/integration/git_environments.rs`: in depth-1 and depth-50 clones of an adopted project, `change new` refuses naming the fetch and leaves no workspace, `change status`, `check` and `audit` name the fetch, and after `git fetch --unshallow origin` they behave as in a full clone; a full clone keeps today's messages; the tester's two shapes name the fetch |
| REQ-cmd-change-044 | `tests/integration/git_environments.rs`: in a shallow clone deep enough for the baseline but not for a change's base commit, text `change status <id>` names the shallow clone and `git fetch --unshallow origin`; in a full clone a corrupt correction ledger keeps its own text |
