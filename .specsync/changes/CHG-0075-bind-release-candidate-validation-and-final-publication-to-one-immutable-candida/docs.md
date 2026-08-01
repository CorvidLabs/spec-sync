---
change: CHG-0075-bind-release-candidate-validation-and-final-publication-to-one-immutable-candida
artifact: docs
---

# Docs

- Update `docs/ci-confidence.md` after implementation with the exact RC branch, marker, promotion,
  retry, and failure-recovery commands.
- Document that ordinary pull requests are Ubuntu-authoritative and that platform qualification
  happens only for immutable release candidates.
- Document how to create a new RC marker after any candidate change and how to inspect SHA-bound
  platform evidence.
- Keep final release guidance explicit: qualify first, create the final tag second, upload last.
