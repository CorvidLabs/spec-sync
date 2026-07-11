---
change: CHG-0012-correct-specsync-5-0-documentation-cli-help-and-hub-deep-links
artifact: context
---

# Context

SpecSync 5.0 is released, but a post-release audit found a few examples that
describe an impossible lifecycle transition, stale CLI help text, incomplete
companion-file wording, and standalone documentation redirects that discard the
requested deep link. The README is already appropriately scoped and needs only
installation and language-profile corrections.

Canonical content under `site/src/content/docs/` remains the source of truth for
the CorvidLabs site mirror. This change corrects that source before the mirror is
updated.
