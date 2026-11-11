---
change: inert-ignored-paths-disclosed-at-policy-load
artifact: context
---

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
