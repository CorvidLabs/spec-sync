---
change: change-check-and-change-audit-say-plainly-when-sdd-json-configures-verification-commands-they-did-not-run-so-a-pass-is
artifact: docs
---

# Docs

- `site/src/content/docs/configuration.md`: explain the text note, the three lists it counts, deduplication, the JSON key, and that no configured command runs.
- `docs/ADOPTING.md`: tell adopters that text check/audit remind them which configured commands were not run and where JSON exposes the list.
- `specs/cmd_change/`: add REQ-cmd-change-unrun-commands-001, testing evidence, a lesson plane, version bump, and Change Log row through the native lifecycle.
- `hi/ship.md`: retain SHIP-1.c.3 verbatim and append the narrowed 3b decision provenance; do not invent a new HI ID.
- `INTENT.md`: update the #893 implementation note to match the narrowed policy choice.
- `CHANGELOG.md`: add the additive disclosure under Unreleased.
