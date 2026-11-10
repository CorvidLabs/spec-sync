# Lesson bundle: import-from-dir-round-4-h1-once-purpose-fallback-takes-prose-only

Material for folding this change's lessons into each affected module's lesson file.
Synthesise from what actually happened below; do not restate the change description.

## What this change was

- **Title**: Import from-dir round 4: H1 once, purpose fallback takes prose only
- **Kind**: BugFix
- **Specs**: importer
- **Paths**: src/importer.rs
- **Acceptance**: the H1 renders exactly once with a blank line after frontmatter; the purpose fallback returns the first prose line of the lead-in, never a numbered list item or fenced code

## Evidence

- Verification commit: `5d1065bb6149c78115a009a3882c71ad7e520232`
- Base commit: `a5b942b2083e405ab7a2379f580e3cf70f33a6da`
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
