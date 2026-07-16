---
change: CHG-0044-harden-canonical-numeric-change-ordering-across-chg-9999-to-chg-10000-and-correc
artifact: plan
---

# Plan

1. Tighten numeric change-sequence parsing to canonical widths without removing five-digit support.
2. Replace full-ID lexicographic successor comparison with fail-closed numeric ordering and a full-ID collision tie-break.
3. Add focused regressions for 9999 to 10000, same-sequence IDs, malformed IDs, and noncanonical widths.
4. Modify `REQ-change-026` through a semantic delta and update its canonical regression evidence.
5. Correct the unreleased 5.1 changelog, comparison headers, and Trust candidate comment without claiming a release.
6. Run the complete native, strict SpecSync, and Trust verification boundary before closing approval.
