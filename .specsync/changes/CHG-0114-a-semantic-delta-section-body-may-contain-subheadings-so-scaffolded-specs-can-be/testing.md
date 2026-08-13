---
change: CHG-0114-a-semantic-delta-section-body-may-contain-subheadings-so-scaffolded-specs-can-be
artifact: testing
---

# Testing

## Strategy

"Accept `###`" could easily mean "accept malformed deltas", so the negative control is not
optional here — it is what separates a fix from a hole. Every assertion is paired.

## Verified end to end against a freshly scaffolded module

The fixture is the #564 repro, built from nothing: `git init`, `specsync init`,
`specsync scaffold greeter`, `change new`, then a delta carrying the generated section
**verbatim**.

| case | before | after |
|---|---|---|
| `## Public API` section as generated (`### Structs & Enums`, `### Traits`, `### Functions`) | `error: invalid delta item heading`, exit 1 | approved, exit 0 |
| `## Dependencies` section as generated (`### Consumes`, `### Consumed By`) | rejected | approved, exit 0 |
| **negative control** — a subheading before any item is opened | rejected | **still rejected**, exit 1 |

The negative control is the load-bearing assertion. A parser that accepted a stray
subheading would silently attach content to nothing, which is worse than the defect being
fixed.

## The message

The rejection now names both valid forms:

> invalid delta item heading `### Structs and Enums` — a delta item must be
> `### REQUIREMENT <id>` or `### SPEC SECTION <name>`; subheadings are only content once an
> item has been opened

The previous message named the offending heading but not what was wrong with it, which is
why this cost a fresh diagnosis on each of five encounters.

## This change is its own test

Authoring it exercises the parser it fixes: the workspace was created, approved, and
verified by the built binary.

## Regression surface

2210 unit and 331 integration tests pass unchanged, including every existing delta in this
repository's archive. The change only widens what is accepted inside an already-open item;
no previously valid delta parses differently.

## Not covered

No unit test asserts the new acceptance directly. Delta parsing has no focused harness in
this change's scope, and the behaviour is better pinned in the sandbox, where a drill can
scaffold a module and drive the whole loop — which is the shape that would have caught this
originally.

## Requirement evidence

| Requirement | Evidence |
|-------------|----------|
| REQ-change-065 | `cargo test` (2210 + 331, 0 failures) plus the three end-to-end cases above against a freshly scaffolded module: both generated sections are accepted verbatim where they were previously rejected, and a subheading before any item is still refused with a message naming both valid forms |
