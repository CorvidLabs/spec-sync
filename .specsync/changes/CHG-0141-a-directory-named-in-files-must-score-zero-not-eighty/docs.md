---
change: CHG-0141-a-directory-named-in-files-must-score-zero-not-eighty
artifact: docs
---

# Docs

A CHANGELOG entry under `## [Unreleased]` → `### Fixed`.

## What it states

- **The disagreement, concretely:** `score --strict` returning 80 and exit 0 on the same spec
  `check` hard-fails — two commands, one spec, opposite verdicts.
- **Why 80 specifically**, because the number looks arbitrary until you see the arithmetic:
  freshness credited 15/15 for a path that exists, the API dimension scored 0 because the read
  failed, and the total landed exactly on the inclusive strict bar.
- **That the bar was not the problem.** The obvious fix — move the threshold — would have been
  wrong; a spec mapping a directory should not be scoring 80.
- **The mechanism:** a directory was reported as `Unreadable`, a category meaning "missing or
  not UTF-8", whose message never said the word directory.
- **That nine files moved for a one-command report**, because the classification is now made
  once and consumed by validator, score, diff, issues, lifecycle and mcp.

## Behaviour change a reader must not be surprised by

A spec whose `files:` entry is a directory now scores **0 (F)** and fails `--strict` and
`--min-score` where it previously could pass. Any project with such a mapping will see a score
drop — which is the correction, not a regression, since `check` was already refusing it.

`check` is unchanged.
