---
change: CHG-0111-report-the-checks-that-frontmatter-parse-failure-prevented-instead-of-printing-a
artifact: testing
---

# Testing

## Strategy

The defect is a false statement, not a missing one, so the tests that matter are the
**controls**: prove the checks genuinely work once they can run. Without them, changing
`✓` to `⊘` could be argued as cosmetic.

## Verified by hand

| fixture | before | after |
|---|---|---|
| no frontmatter, body has 2 of 8 required sections, project schema has tables | four green lines, exit 1 | four `⊘ … skipped (frontmatter invalid)` lines, exit 1 |
| **control A** — same body, valid frontmatter | — | **5 missing-section errors**, exit 1 |
| **control B** — valid frontmatter, `db_tables: [users, ghosts]`, schema has only `users` | — | `✗ DB table not found in schema: ghosts`, exit 1 |

Control B is the one that matters most for this revision. The DB-table line was missed by an
earlier attempt because its guard tests whether the *project schema* has tables rather than
whether the spec's `db_tables:` is readable, so it did not match the shape of the other
three. Control B establishes that the check is real and was simply never reached.

## Regression surface

The change adds a branch ahead of four existing ones and leaves every valid-frontmatter path
untouched. The suite guards against the branch being taken too eagerly: 2210 unit and 331
integration tests pass unchanged, including this repository's own 62 specs, none of which
have invalid frontmatter.

## Not covered

No unit test asserts the new wording directly. The renderer has no test harness in this
change's scope, and output wording is pinned behaviourally in the sandbox — a drill
assertion alongside the existing `⊘` coverage in drill 040 is the right home.

## Requirement evidence

| Requirement | Evidence |
|-------------|----------|
| REQ-commands-008 | `cargo test` (2210 + 331, 0 failures) plus the three hand-verified fixtures above: invalid frontmatter reports four skipped checks instead of four green ones, control A proves the section check reports five genuine failures once it runs, and control B proves the DB-table check reports a missing table once it runs. Exit status is 1 before and after, confirming the gate was never the problem — only the report |
