---
change: make-it-obvious-on-a-pull-request-when-a-change-is-still-open-and-when-it-is-archived
artifact: research
---

# Research

The preflight in `.github/workflows/ci.yml` fails only when an active change's verification commit is missing from the branch. A current open change passes, and so does an archived one. `implementation-gate` is the required rollup. A new always-on job has to be in its needs list and its GATES rows, or the required-gate test fails.

`change ship` finalizes in `run_ship` and commits only when `--push` is set, inside `ship_commit_and_push_archive`.
