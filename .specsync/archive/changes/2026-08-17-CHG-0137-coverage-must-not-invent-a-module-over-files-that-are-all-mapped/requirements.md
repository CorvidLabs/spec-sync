---
change: CHG-0137-coverage-must-not-invent-a-module-over-files-that-are-all-mapped
artifact: requirements
---

# Requirements

Two requirements are added as semantic deltas, one per affected module. The delta files are the
source; `specs/` is materialized from them rather than hand-edited.

## `deltas/validator.md` — REQ-validator-041

The behavioural rule. Coverage may report a module as lacking a spec only on evidence drawn from
that module's own files, never on the absence of a spec directory bearing its name.

Its final acceptance criterion is the one that matters most: **every derivation path applies the
same rule**. A requirement written only about flat-file stems would have been satisfied by a fix
that left the manifest derivation — the one producing the phantom on this repository — untouched.

## `deltas/manifest.md` — REQ-manifest-018

The enabling fact. A manifest module must carry the source paths it declares, so a consumer can
judge it against its own files rather than its name.

Stated separately because it is a real contract change in a different module, not an
implementation detail of the validator fix. `source_paths` existed but was dead code; a
requirement is what makes it load-bearing rather than incidental.

## Explicitly retained behaviour

Both requirements carry criteria for what must **not** change:

- A module owning at least one unmapped file is still reported.
- A module owning no discovered file at all is still reported.
- Coverage percentages are unaffected.

The second is the vacuity control. Without it, deleting the feature satisfies the rest.
