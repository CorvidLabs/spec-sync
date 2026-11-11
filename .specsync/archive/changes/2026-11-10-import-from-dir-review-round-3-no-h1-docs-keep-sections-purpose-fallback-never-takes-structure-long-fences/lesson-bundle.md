# Lesson bundle: import-from-dir-review-round-3-no-h1-docs-keep-sections-purpose-fallback-never-takes-structure-long-fences

Material for folding this change's lessons into each affected module's lesson file.
Synthesise from what actually happened below; do not restate the change description.

## What this change was

- **Title**: Import --from-dir review round 3: no-H1 docs keep sections, purpose fallback never takes structure, long fences
- **Kind**: BugFix
- **Specs**: importer
- **Paths**: src/importer.rs, tests/
- **Acceptance**: a markdown source with no H1 keeps its sections and lead paragraphs; the purpose fallback never emits a heading, table row, fence or list marker; four-backtick fences nest three-backtick fences without splitting sections

## Evidence

- Verification commit: `799d3723829b68fc5d682d0a627a1584ca173797`
- Base commit: `63319e1bd96c3bdd9e1049d3298b42ed04825f89`
- Verified by: `specsync check --spec importer`

## From the change's context.md

# Context

filled

filled

filled

## From the change's testing.md

# Testing

filled

## Where these lessons go

- `specs/importer/importer.3md`
