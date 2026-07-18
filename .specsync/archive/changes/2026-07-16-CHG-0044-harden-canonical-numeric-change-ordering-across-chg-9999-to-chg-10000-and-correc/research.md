---
change: CHG-0044-harden-canonical-numeric-change-ordering-across-chg-9999-to-chg-10000-and-correc
artifact: research
---

# Research

`change_sequence` already parses arbitrarily wide decimal fields, and most CHG43 succession ordering uses `(numeric sequence, full ID)`. The remaining `accepted_change_has_current_canonical_successors` predicate compares `candidate.id > record.id`, which reverses the 9999-to-10000 boundary. Its timestamp filter cannot repair the identity-order bug and is not succession proof.

The parser's current minimum-width check admits `CHG-09999-*` as sequence 9999. Canonical rendering provides a small, deterministic validation rule that rejects alias spellings and numeric overflow before comparison.

CHG37's archived evidence and `REQ-exports-003` confirm that extensionless export-star targets resolve sibling `.mjs` and `.cjs` files in both regex and AST modes. The current changelog bullet mentions only discovery and coverage, so the release description is incomplete. Repository tags confirm no Trust v1.0.1 release exists yet, so the workflow comment must describe its exact SHA as an unreleased candidate rather than a released version.
