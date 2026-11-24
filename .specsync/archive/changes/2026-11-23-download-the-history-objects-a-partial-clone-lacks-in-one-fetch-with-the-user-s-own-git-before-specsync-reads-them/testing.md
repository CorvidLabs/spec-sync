---
change: download-the-history-objects-a-partial-clone-lacks-in-one-fetch-with-the-user-s-own-git-before-specsync-reads-them
artifact: testing
---

# Testing

## Requirement evidence

| Requirement | Evidence |
|---|---|
| REQ-change-172 | `tests/integration/git_environments.rs`, with a blobless clone whose remote resolves only through the user's global `insteadOf`: `change review`, `status`, `audit` and `ship-status` succeed after one fetch with the stderr line, and the objects are present; with the user's configuration gone, the refusal names a `git fetch … <oid> …` that, run as printed, lets the command succeed; a full clone makes no fetch; a blobless clone on main with an active change audits #1081's base-commit requirements after the one fetch |
