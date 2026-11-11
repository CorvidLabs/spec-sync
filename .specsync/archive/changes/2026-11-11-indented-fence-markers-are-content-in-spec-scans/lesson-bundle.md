# Lesson bundle: indented-fence-markers-are-content-in-spec-scans

Material for folding this change's lessons into each affected module's lesson file.
Synthesise from what actually happened below; do not restate the change description.

## What this change was

- **Title**: Indented fence markers are content in spec scans
- **Kind**: BugFix
- **Specs**: cmd_check, parser
- **Paths**: src/commands/check.rs, src/parser.rs, tests/integration/fix.rs
- **Acceptance**: a spec with a 4-space-indented fence marker before its export table has its exports read and --fix adds each undocumented export exactly once to the real table

## Evidence

- Verification commit: `707dfcf63fce28feee67558a9304f5c5846d6972`
- Base commit: `46f55652649a72ddf5e1715938b504b96686566b`
- Verified by: `specsync check --spec cmd_check --spec parser`

## From the change's context.md

# Context

#768 re-cut per t370: fence fix only (the table-selection reorder is dropped — t384 showed it contradicts REQ-cmd-check-017). Both scanners (export_table in check.rs, blank_fenced_code in parser.rs) trimmed the indent before the fence check; a 4-space-indented fence marker opened a phantom fence and the real table read as fenced, so --fix appended a duplicate row. Both now check indentation first. Case 1 of the issue stays open (needs a requirements decision).

## From the change's design.md

# Design

Two scanners share one rule: a line indented four or more spaces (or a tab) is
content under CommonMark and never updates fence state. `export_table`
(src/commands/check.rs) checks indentation before the fence-marker match;
`parser::blank_fenced_code` (src/parser.rs) computes the marker only for
unindented lines, so an indented line inside an open fence is still blanked as
fence content. No new types, no format changes; the regression test drives
`check --fix` end to end.

## From the change's testing.md

# Testing

## Requirement evidence

| Requirement | Evidence |
|---|---|
| REQ-cmd-check-017 | fix_treats_an_indented_fence_marker_as_content |
| REQ-parser-004 | same test through blank_fenced_code |

## Where these lessons go

- `specs/cmd_check/cmd_check.3md`
- `specs/parser/parser.3md`
