---
change: CHG-0044-harden-canonical-numeric-change-ordering-across-chg-9999-to-chg-10000-and-correc
artifact: design
---

# Design

Keep `change_sequence` as the single numeric parser, but require its digit field to equal the canonical rendering of the parsed value: four zero-padded digits below 10000 and an unpadded decimal string at or above 10000. Parsing failure, short widths, redundant wider leading zeroes, overflow, and non-ASCII/non-digit content return `None`.

Add one comparison helper that returns false unless both IDs parse canonically. Compare `(numeric sequence, full ID)` tuples so `CHG-10000-*` follows `CHG-9999-*` and acknowledged same-sequence IDs remain deterministic. Route the remaining canonical-successor predicate through that helper instead of comparing strings.

The code regression covers the numeric-width boundary, same-sequence tie-break, malformed IDs, and noncanonical wider zero-padding. Documentation changes describe already-implemented behavior and present release state accurately: SpecSync 5.1.0 is released before Trust 1.0.1, and the pinned Trust SHA remains an unreleased candidate until that later release exists.
