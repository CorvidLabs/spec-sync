---
change: CHG-0066-make-issue-427-spec-merge-resolution-lossless-and-truthful-by-parsing-diff3-base
artifact: design
---

# Design

## Region Model

Parse conflict input into clean segments and typed conflict regions. A region retains ours,
optional diff3 base, incoming, and both labels. Only exact standard marker forms are accepted.
Missing delimiters and orphan, duplicate, nested, incomplete, or lookalike markers produce a
manual result with truthful outer labels.

## Resolution Rules

- Ignore the base section as an output candidate; use it only to identify diff3 structure.
- Union and deterministically sort supported frontmatter lists.
- Parse versions numerically and choose the maximum.
- Resolve identical scalar/table values; keep divergent values manual.
- Merge changelog rows only when the union is lossless.
- Keep one-sided scalars, table headers/separators, and nested YAML mappings manual.

## Persistence

Build the candidate output in memory, validate its frontmatter with the checked YAML parser and
canonical required-field/status rules, and write only if every region resolved. A manual region,
parse error, duplicate key, or validation error returns details while retaining the original
bytes.

## Diagnostics

Preserve raw side labels and report which values were retained or why resolution stopped.
Messages must not call incoming content HEAD or otherwise invert side identity.
