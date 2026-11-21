---
id: correct-the-readme-and-documentation-for-specsync-6-4-history-fragments-the-archive-commit-diagrams-and-output-samples
state: archived
type: documentation
base_commit: 3f0985e80acab0fe4559fe322cf929be40057414
---

# Correct the README and documentation for SpecSync 6.4: history fragments, the archive commit, diagrams and output samples

## Intent

Correct the README and documentation for SpecSync 6.4: history fragments, the archive commit, diagrams and output samples

## Affected Canonical Specs

- None

## Acceptance Criteria

- (1) The 21 items in defs/docs-6.4-accuracy-definition.md are corrected as written, with no other wording changed. (2) The command check script reports nothing new. (3) Each output sample changed is the output of a real run. (4) cd site && bun test passes, Mermaid and lifecycle checks included, and bun run build passes. (5) validate-release-version.py and specsync check --strict pass.

## No-spec Rationale

Documentation only: no contract, requirement or code changes
