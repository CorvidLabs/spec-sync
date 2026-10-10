---
change: make-it-obvious-on-a-pull-request-when-a-change-is-still-open-and-when-it-is-archived
artifact: testing
---

# Testing

REQ-cmd-change-019

- `.github/scripts/test-archive-readiness.sh` covers an open change, several open changes, and no open changes.
- `ship_commits_the_archive_and_names_merge_as_what_is_left` in `src/commands/change.rs`.
- `status_says_which_open_change_to_archive_first` in `src/commands/change.rs`.
- `python3 .github/scripts/test-required-ci-gate.py` still passes with the Archive job on the required gate.
