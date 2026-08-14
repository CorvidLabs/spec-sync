---
id: CHG-0124-a-source-file-or-spec-body-carrying-an-unresolved-merge-conflict-must-be-refused
state: implementing
type: bug_fix
base_commit: e29e3b0dd9fe302b92152283dc9c3898f47b632f
---

# A source file or spec body carrying an unresolved merge conflict must be refused, because extracting declarations from both sides of a hunk describes source that does not exist

## Intent

A source file or spec body carrying an unresolved merge conflict must be refused, because extracting declarations from both sides of a hunk describes source that does not exist

## Affected Canonical Specs

- `merge`
- `exports`
- `validator`
- `cmd_merge`
- `cmd_diff`
- `scoring`

## Acceptance Criteria

- A source file whose extractor read declarations from both sides of an unresolved conflict hunk is refused, naming which side contributed which symbols, instead of comparing the union against the spec and passing. A spec body carrying conflict markers is refused before frontmatter parsing, so the failure is named as a conflict rather than as an incidental duplicate key. A path git reports as unmerged is refused whatever the extractor made of the bytes, and a git failure reports unknown rather than clean. Conflict-marker syntax inside a fenced code block in a spec is not a conflict. spec-sync's own repository, which contains twelve well-formed conflict triples inside test string literals, still passes check --strict.

## No-spec Rationale

Not applicable
