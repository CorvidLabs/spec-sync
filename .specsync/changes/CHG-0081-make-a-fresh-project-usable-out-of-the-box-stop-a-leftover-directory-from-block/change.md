---
id: CHG-0081-make-a-fresh-project-usable-out-of-the-box-stop-a-leftover-directory-from-block
state: implementing
type: bug_fix
base_commit: 9e431ec5f0d21e8e2f14dc52d512a71566447cf4
---

# Make a fresh project usable out of the box, stop a leftover directory from blocking change new, and extract a lock-free verification body

## Intent

Make a fresh project usable out of the box, stop a leftover directory from blocking change new, and extract a lock-free verification body

## Affected Canonical Specs

- `change`

## Acceptance Criteria

- specsync init detects a test command for go.mod, pyproject.toml/pytest.ini and package.json in addition to Cargo, bun, Swift and fledge, and warns at init time naming the file and an example when none is detected rather than silently writing an empty list; a change directory containing no state.json is skipped as not-active-here by both active-change read paths instead of failing closed, so change new succeeds on a branch that does not contain an earlier change and audit --strict no longer reports the sequence ledger as uncovered, while every other read error still fails closed; verify_change_with_strict acquires the project lock and delegates to a lock-free verify_change_locked documented as requiring the caller to hold it, with no behaviour change

## No-spec Rationale

Not applicable
