---
id: a-closed-output-never-crashes-specsync-or-hides-its-exit-status
state: archived
type: bug_fix
base_commit: 46f55652649a72ddf5e1715938b504b96686566b
---

# A closed output never crashes specsync or hides its exit status

## Intent

a closed output never crashes specsync or hides its exit status

## Affected Canonical Specs

- `cli`
- `watch`

## Acceptance Criteria

- With its stdout (or stderr) reader gone, specsync drops further output to that stream and exits with the command's own status, 0 or 1, never a panic or exit 101; watch stops once nobody reads its output; any other write error still panics

## No-spec Rationale

Makes behavior match REQ-cli (exit 0 on success, 1 on errors); the specs gain only error-case rows
