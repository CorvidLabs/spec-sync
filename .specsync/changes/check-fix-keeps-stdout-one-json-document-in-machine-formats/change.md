---
id: check-fix-keeps-stdout-one-json-document-in-machine-formats
state: draft
type: bug_fix
base_commit: 18385ef5ead84f0b51d70a29a6fed8a3e5ca222a
---

# Check --fix keeps stdout one JSON document in machine formats

## Intent

check --fix keeps stdout one JSON document in machine formats

## Affected Canonical Specs

- `cmd_check`

## Acceptance Criteria

- check --fix --json and check --strict --force --fix --dry-run --json print exactly one JSON document on stdout, and each per-spec added or would-add line on stderr; text output is unchanged

## No-spec Rationale

cmd_check's acceptance criteria say JSON output is a single object with passed, errors, warnings, stale and specs_checked; check --fix and --fix --dry-run printed per-spec progress lines on stdout before it (#475)
