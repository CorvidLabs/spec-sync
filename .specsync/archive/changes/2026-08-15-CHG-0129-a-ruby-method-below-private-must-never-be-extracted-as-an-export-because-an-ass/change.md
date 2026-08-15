---
id: CHG-0129-a-ruby-method-below-private-must-never-be-extracted-as-an-export-because-an-ass
state: archived
type: bug_fix
base_commit: e0af129cce77d654ad2d1de42310932fe42c4780
---

# A Ruby method below private must never be extracted as an export, because an assignment-form conditional desynced the visibility stack and published private methods as contract

## Intent

A Ruby method below private must never be extracted as an export, because an assignment-form conditional desynced the visibility stack and published private methods as contract

## Affected Canonical Specs

- `exports`

## Acceptance Criteria

- A Ruby method sitting below a private keyword is never reported as an export, whether it precedes or follows an assignment-form multi-line conditional. Documenting such a method is an orphan error rather than an accepted export, so a repository cannot silence the warning by publishing a private method as contract. A statement-form conditional, which never triggered the desync, behaves exactly as before. Public methods above private continue to be extracted.

## No-spec Rationale

Not applicable
