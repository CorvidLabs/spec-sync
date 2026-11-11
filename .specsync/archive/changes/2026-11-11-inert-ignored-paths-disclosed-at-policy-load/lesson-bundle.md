# Lesson bundle: inert-ignored-paths-disclosed-at-policy-load

Material for folding this change's lessons into each affected module's lesson file.
Synthesise from what actually happened below; do not restate the change description.

## What this change was

- **Title**: Inert ignored paths disclosed at policy load
- **Kind**: BugFix
- **Specs**: change
- **Paths**: src/change/sdd_version.rs, src/change/validate_loaded_change.rs, src/change_tests.rs, src/change_tests/, tests/, .specsync/sdd.json
- **Acceptance**: a policy whose ignored_paths lists the specs tree or a protected policy file prints a load-time note naming the inert entry and its removal; adopt no longer writes the inert specs/ entry

## Evidence

- Verification commit: `8cc1b3094d126222132c28041d8ab51dc76aca42`
- Base commit: `46f55652649a72ddf5e1715938b504b96686566b`
- Verified by: `specsync check --spec change`

## From the change's context.md

# Context

Issue #676: an `ignored_paths` entry that tries to cover the specs tree or an
SDD policy file is silently overridden — the override is disclosed only at
audit, three verbs too late. The default policy itself writes one such inert
entry (`specs/`), so the disclosure must (a) fire at policy load where the
conflict exists, and (b) stop being written by `adopt`/`init`.

Rule: warn only for entries whose coverage is ENTIRELY always-meaningful
("specs/" or anything inside the specs tree; a protected policy file named
directly). `.specsync/` stays silent because it also covers the change
workspaces, where it does real work. The note goes to stderr, never stdout, so
`--json` payloads stay clean.

## From the change's testing.md

# Testing

Unit tests on the detector (specs tree entry, protected file, .specsync/
silent); integration test that `change status` prints the note on stderr.

## Requirement evidence

| Requirement | Evidence |
|---|---|
| REQ-change-155 | change_tests + integration assertions |

## Where these lessons go

- `specs/change/change.3md`
