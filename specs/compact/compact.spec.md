---
module: compact
version: 8
status: stable
files:
  - src/compact.rs
db_tables: []
tracks: [94]
depends_on:
  - specs/validator/validator.spec.md
---

# Compact

## Purpose

Reduces changelog table size in spec files by keeping only the last N entries and summarizing older ones into a single compacted row with a date range and entry count.

## Public API

### Exported Functions

| Function | Parameters | Returns | Description |
|----------|-----------|---------|-------------|
| `compact_changelogs` | `root: &Path, specs_dir: &Path, keep: usize, dry_run: bool` | `CompactReport` | Plan, stage, and compact changelog tables while reporting every failure |
| `complete` | `&self` | `bool` | Whether the invocation recorded no failures |
| `partial` | `&self` | `bool` | Whether an incomplete apply published at least one planned replacement |

### Exported Structs

| Type | Description |
|------|-------------|
| `CompactResult` | Planned result for one spec, including truthful counts and whether publication was applied |
| `CompactFailure` | Structured path, operation, and error for a failed read, parse, stage, or publish |
| `CompactReport` | Invocation outcome with planned/succeeded counts, results, failures, and complete/partial predicates |

## Invariants

1. Only an exact `## Change Log` H2 outside fenced and indented code is processed
2. Only the first contiguous, width-valid table outside fenced and indented code in the section is compacted; later tables are preserved
3. The last `keep` data rows are preserved; earlier rows are summarized
4. Summary row contains the date range of compacted entries and their count
5. If a changelog has fewer than `keep + 1` entries, no compaction occurs
6. `dry_run: true` returns results without modifying files
7. Handles both 2-column and 3+ column tables with appropriate summary format
8. Re-running compaction with the same `keep` value is byte-for-byte idempotent.
9. Escaped table pipes, code-span pipes, every original LF/CRLF line terminator, and the original final-newline state are preserved
10. Only rows carrying the exact `<!-- specsync:compact:v1 -->` provenance marker are folded as prior summaries
11. Multiple marked summaries, malformed table widths, and fixed-width count overflow fail closed
12. Apply mode preflights every replacement and stages same-directory temporary files before publication
13. Staging failures retain every planned result/count with zero writes; late publish failures retain all results and report exact partial progress
14. Indented pipe code terminates table data, indented separators fail before writes, and a generated summary is valid only as the first data row

## Behavioral Examples

### Scenario: Compact a long changelog

- **Given** a spec with 20 changelog entries and `keep = 5`
- **When** `compact_changelogs` is called
- **Then** the first 15 entries are replaced with one marked summary row ending in `Compacted: 15 entries <!-- specsync:compact:v1 -->`

### Scenario: Re-run an already compacted changelog

- **Given** a changelog has one generated compaction summary and the latest five ordinary rows
- **When** `compact_changelogs` runs again with `keep = 5`
- **Then** no rows are removed and the file remains byte-for-byte unchanged

### Scenario: Short changelog (no compaction needed)

- **Given** a spec with 3 changelog entries and `keep = 5`
- **When** `compact_changelogs` is called
- **Then** the spec is skipped (not included in results)

### Scenario: Dry run

- **Given** specs with long changelogs
- **When** `compact_changelogs(root, specs_dir, 5, true)` is called
- **Then** returns `CompactResult` entries but does not modify any files

## Error Cases

| Condition | Behavior |
|-----------|----------|
| Spec file unreadable | Records a structured failure, exits nonzero at the command boundary, and performs no writes |
| Multiple marked summaries or malformed table | Records a parse failure and performs no writes |
| Indented separator or reordered generated summary | Records a parse failure and performs no writes |
| Staging failure | Records the failing path/operation and performs no writes |
| Late atomic publish failure | Reports an incomplete/partial outcome and never claims complete success |
| No changelog section found | Spec is silently skipped |

## Dependencies

### Consumes

| Module | What is used |
|--------|-------------|
| validator | `find_spec_files` to locate all spec files |

### Consumed By

| Module | What is used |
|--------|-------------|
| main | `compact_changelogs` via `cmd_compact` subcommand |

## Change Log

| Date | Change |
|------|--------|
| 2026-07-26 | Stop before indented pipe-code and reject indented separators or reordered generated summaries before mutation |
| 2026-07-26 | Close adversarial #417 gaps: ignore fenced/prefix headings, preserve keep-zero EOF bytes, retain complete plan counts on staging failure, and characterize late partial publication |
| 2026-07-26 | Harden #417 after adversarial review: provenance-mark summaries, preserve exact line endings, parse contiguous tables safely, reject ambiguity/overflow, and report staged atomic-write failures |
| 2026-07-26 | Fix #417: make summary folding idempotent and exact, preserve escaped pipes and trailing newlines, and report truthful counts |
| 2026-04-10 | Populated requirements.md with user stories, acceptance criteria, constraints, and out-of-scope items |
| 2026-04-06 | Initial spec for v3.3.0 |
| 2026-07-11 | CHG-0010-canonicalize-every-specsync-5-0-contract-and-requirement: Canonicalize every SpecSync 5.0 contract and requirement |
| 2026-07-27 | CHG-0065-make-issue-417-changelog-compaction-idempotent-and-provide-truthful-portable-str: Make issue 417 changelog compaction idempotent and provide truthful portable structured maintenance output |
