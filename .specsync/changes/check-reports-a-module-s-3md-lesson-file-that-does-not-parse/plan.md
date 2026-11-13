---
change: check-reports-a-module-s-3md-lesson-file-that-does-not-parse
artifact: plan
---

# Plan

## Approach

One companion check in the validator: parse an existing `<module>.3md` and warn when it does not
parse.

## Out of scope

Briefing output (REQ-change-106 keeps its silent fallback) and repairing a file.

## Steps

1. A failing integration test: a duplicate-`z` `.3md` gets the warning and fails `--strict`; a
   parsing one and none pass.
2. The check in `validate_spec_content_internal`.
3. The CHANGELOG line.

## Risks

A repository with an unparseable `.3md` today now fails `check --strict`. That is the point; the
warning names the file and the error, and this repository's only instance was fixed by #1062.

## Constraints consulted

REQ-change-106: an unreadable or invalid 3md file does not fail a lifecycle command.
