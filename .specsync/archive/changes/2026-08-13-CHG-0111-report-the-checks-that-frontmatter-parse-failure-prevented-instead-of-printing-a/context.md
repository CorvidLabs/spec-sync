---
change: CHG-0111-report-the-checks-that-frontmatter-parse-failure-prevented-instead-of-printing-a
artifact: context
---

# Context

## What led here

CorvidLabs/spec-sync#553. A spec whose frontmatter cannot be parsed reported **four** checks
as passing that never ran:

```
✗ Frontmatter invalid
  ✗ Missing or malformed YAML frontmatter (expected --- delimiters)
✓ All source files exist
✓ All DB tables exist in schema
✓ All required sections present
✓ All dependency specs exist
```

That spec's body contained two of the eight default required sections.

## Why this one is worse than the rest of the series

The other findings in this class are **vacuous** — they claim success over an empty set.
This one is **false**. `✓ All required sections present` is a wrong answer to a question,
not an unasked question reported as passed. Verified with a control: the identical body
with valid frontmatter reports **five** missing required sections.

## The fourth line, and how it was nearly missed

An earlier attempt at this fix (PR #556, closed) patched three of the four. The DB-table
line was skipped because it does not resemble the others:

```rust
} else if !schema_input.table_names.is_empty() {
    println!("✓ All DB tables exist in schema")
}
```

Its guard tests whether the **project schema** has tables — not whether the spec's
`db_tables:` could be read — so pattern-matching on the shape of the other three stepped
straight over it. It was found only by sweeping the codebase for affirmative claims
deliberately, rather than by waiting for the next report.

Control for that line: the same spec with valid frontmatter and `db_tables: [users, ghosts]`
correctly reports `✗ DB table not found in schema: ghosts`. The check works; it simply
never ran and said it had.

## Root cause

All four infer success from the **absence of errors in their category**. Unparseable
frontmatter yields no `files:`, no `db_tables:`, no `depends_on:`, and no section
validation, so each category is empty for want of *input* rather than for want of
*problems*. Absence of evidence rendered as evidence of absence.

## What a session picking this up needs to know

The vocabulary already existed: the draft path prints
`⊘ Section validation skipped (status: draft)`. This reuses that shape rather than
inventing a second one.

This is the fifth instance of the same class this cycle (#546, #547, #548, #550, #553). A
sweep of the remaining affirmative claims is in progress and has already produced #558.
