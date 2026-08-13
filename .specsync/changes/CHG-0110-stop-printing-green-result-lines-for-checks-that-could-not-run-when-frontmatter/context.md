---
change: CHG-0110-stop-printing-green-result-lines-for-checks-that-could-not-run-when-frontmatter
artifact: context
---

# Context

## What led here

CorvidLabs/spec-sync#553, found during stress testing and re-verified by hand against
product main before being acted on.

A spec with no frontmatter at all, whose body contains only `## Purpose` and
`## Public API`, reported:

```
✗ Frontmatter invalid
  ✗ Missing or malformed YAML frontmatter (expected --- delimiters)
✓ All source files exist
✓ All required sections present
✓ All dependency specs exist
```

Six of the eight default required sections were absent.

## Why this is worse than the other findings in this series

The rest of the "green result for work that did not happen" family are **vacuous**: they
claim success over an empty set. This one is **false**. `✓ All required sections present`
is a wrong answer to a question, not an unasked question reported as passed.

Verified with a control: the identical spec body with valid frontmatter reports **five**
missing required sections. The sections really were missing; the tool really did say they
were present.

The exit code was already correct — 1 — so this never let a bad spec through a gate. What
was wrong was the report. An author who fixes the frontmatter first is then surprised by
errors that were always there, and an agent consuming the output has no way to know three
of those lines were worthless.

## Root cause

All three checks infer success from the **absence of errors in their category**:

```rust
if file_errors.is_empty() { println!("✓ All source files exist") }
if section_errors.is_empty() { println!("✓ All required sections present") }
if dep_errors.is_empty() { println!("✓ All dependency specs exist") }
```

When frontmatter cannot be parsed there are no `files:` and no `depends_on:`, and section
validation never runs, so each category is empty for want of input rather than for want of
problems. Absence of evidence was being rendered as evidence of absence.

## What a session picking this up needs to know

The vocabulary for "this did not run" already existed: the draft path prints
`⊘ Section validation skipped (status: draft)`. This change reuses that shape rather than
inventing a second one, so there is one way to say it.
