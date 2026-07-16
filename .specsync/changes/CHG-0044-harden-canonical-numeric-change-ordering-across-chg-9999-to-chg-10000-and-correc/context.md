---
change: CHG-0044-harden-canonical-numeric-change-ordering-across-chg-9999-to-chg-10000-and-correc
artifact: context
---

# Context

Change IDs use a minimum four-digit numeric sequence and intentionally support values beyond 9999. One legacy canonical-successor predicate still compares complete ID strings, so `CHG-10000-*` sorts before `CHG-9999-*` lexicographically even though its numeric sequence is later. The shared parser also accepts wider encodings with redundant leading zeroes, allowing a noncanonical spelling to enter numeric ordering.

CHG44 makes successor ordering numeric and fail closed: both IDs must have canonical numeric encodings, numeric sequence is primary, and the full canonical ID is used only to order acknowledged same-sequence collisions. This is separate from CHG43's signed successor-evidence graph and preserves both lifecycle histories.

The unreleased 5.1 changelog already describes `.mjs`/`.cjs` discovery and coverage, but omits the accepted CHG37 behavior that extensionless export-star targets in module-JavaScript barrels resolve sibling `.mjs` and `.cjs` modules. Two comparison tables also retain stale 5.0 headers. The Trust workflow pins an immutable candidate commit but labels it as v1.0.1 even though that tag does not exist. CHG44 corrects these release-facing descriptions without tagging or publishing anything.
