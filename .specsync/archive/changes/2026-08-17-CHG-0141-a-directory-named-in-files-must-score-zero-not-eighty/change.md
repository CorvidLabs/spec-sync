---
id: CHG-0141-a-directory-named-in-files-must-score-zero-not-eighty
state: archived
type: bug_fix
base_commit: adbfb442e6ea73d78f4d8d4dc830ad1077c7b961
---

# A directory named in files: must score zero, not eighty

## Intent

a directory named in files: must score zero, not eighty

## Affected Canonical Specs

- `exports`
- `scoring`
- `validator`
- `cli_args`
- `mcp`
- `cmd_diff`
- `cmd_score`
- `cmd_issues`
- `cmd_lifecycle`

## Acceptance Criteria

- a spec whose files: entry is a directory scores 0 with grade F and exits 1 under --strict, where it previously scored exactly 80 and passed; the message names the word directory rather than reporting missing or not UTF-8; check continues to hard-fail the same mapping unchanged, so score and check agree; a spec naming a real source file still scores 100 and exits 0, which is the vacuity control that stops a score-everything-zero fix from passing; the directory classification is made once in the export scan and consumed by validator, score, diff, issues, lifecycle and mcp, so a directory cannot be classified two ways across commands.

## No-spec Rationale

Not applicable
