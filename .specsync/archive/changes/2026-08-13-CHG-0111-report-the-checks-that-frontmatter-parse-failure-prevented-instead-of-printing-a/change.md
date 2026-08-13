---
id: CHG-0111-report-the-checks-that-frontmatter-parse-failure-prevented-instead-of-printing-a
state: archived
type: bug_fix
base_commit: c7977425ad791f6eef99af5a29032c55532f84fb
---

# Report the checks that frontmatter parse failure prevented instead of printing a green line for each

## Intent

Report the checks that frontmatter parse failure prevented instead of printing a green line for each

## Affected Canonical Specs

- `commands`

## Acceptance Criteria

- When a spec's frontmatter cannot be parsed, `specsync check` reports the source-file, DB-table, required-section, and dependency checks as skipped rather than printing a green result line for each of the four. A spec whose frontmatter is valid continues to report all four normally: the same body with valid frontmatter still reports every genuinely missing required section, and a declared table absent from the schema is still reported as missing. Exit status is unchanged — invalid frontmatter remains an error.

## No-spec Rationale

Not applicable
