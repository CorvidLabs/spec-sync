---
change: CHG-0141-a-directory-named-in-files-must-score-zero-not-eighty
artifact: design
---

# Design

`ExportScan` gains a `Directory` variant, decided by a shared `files_entry_is_directory`
predicate **before** `read_to_string` is attempted. A directory therefore never reaches the code
path that produces `Unreadable`.

Every consumer of the scan handles the new variant explicitly: validator, scoring's API and
freshness dimensions, diff, issues, lifecycle and mcp. The compiler enforces that, which is the
reason to model this as a variant rather than as a boolean checked at each call site — a new
consumer cannot silently inherit the old collapse.

## Why classify rather than special-case scoring

Scoring alone could have been taught to check `is_dir()` before scoring the API dimension. That
would fix the reported symptom and leave `diff` and the export scan still reporting a directory
as unreadable, which is the sibling-site pattern this codebase has produced eight times.

Classifying once and consuming everywhere means the question "is this a directory" has exactly
one answer in the process.

## Score zero, do not hard-fail

`score` stays a metric. A hard failure would make `--explain` and the JSON payload unavailable
for precisely the spec a user is trying to understand. Zero with grade F is below every strict
and minimum-score floor, so the gate closes without the diagnostic disappearing.

`check` is unchanged: it already hard-failed this mapping and should continue to.

## The vacuity control

`real_source_file_still_scores_at_or_above_strict_bar` passes on both the unfixed and fixed
binaries. Without it, a change that scored everything zero would satisfy the headline assertion
while destroying the command.
