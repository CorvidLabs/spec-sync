---
id: CHG-0108-stop-reporting-success-for-checks-that-did-not-happen-gate-drafts-that-document
state: archived
type: bug_fix
base_commit: 80324cb50220c05ffdf4158eeadb3b9acca1e24b
---

# Stop reporting success for checks that did not happen: gate drafts that document a contract over present source, drop cold-cache drift noise, and stop taking quoted frontmatter paths literally

## Intent

Stop reporting success for checks that did not happen: gate drafts that document a contract over present source, drop cold-cache drift noise, and stop taking quoted frontmatter paths literally

## Affected Canonical Specs

- `parser`
- `hash_cache`
- `cmd_check`
- `commands`
- `types`
- `validator`
- `change`

## Acceptance Criteria

- A spec that is `status: draft`, whose mapped source files exist, and whose Public API names at least one symbol produces a warning, so bare `specsync check` still exits 0 and `specsync check --strict` exits 1. A draft whose mapped files do not exist yet still passes `--strict` unchanged, and so does a draft with an empty Public API. A fresh clone with no hash cache reports no requirements-drift warnings, while a real edit to a companion against a known baseline still reports exactly one. A quoted entry in frontmatter resolves to the path inside the quotes for `files:`, `depends_on:`, `db_tables:` and scalars, an unterminated quote is a frontmatter error, and the hash cache keys the same path the parser resolved. The coverage-gate remediation names at most twelve paths and summarizes the rest.

## No-spec Rationale

Not applicable
