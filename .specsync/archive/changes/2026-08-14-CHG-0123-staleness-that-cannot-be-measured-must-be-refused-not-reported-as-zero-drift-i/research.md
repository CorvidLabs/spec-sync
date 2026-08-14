---
change: CHG-0123-staleness-that-cannot-be-measured-must-be-refused-not-reported-as-zero-drift-i
artifact: research
---

# Research

The four readers were enumerated by grepping the primitives rather than the
symptom: `git_commits_since`, `git_last_commit_hash`, `spec_baseline`,
`is_git_repo`, `has_commits`, `last_commit_hash`. Searching for the reported
behaviour would have found `report` alone.

`scoring.rs:606` was found not by that sweep but by a reviewer measuring grades
across two trees. Worth recording: the sweep found the three readers that print
the word "stale"; the fourth expressed the same computation as a score, and only
a behavioural comparison surfaced it.

`git_utils.rs:43-51` degrades a per-file `git rev-list` failure to zero and is
deliberately tested that way. That is a narrower, accepted case than the two
repo-wide states fixed here, and is left alone.
