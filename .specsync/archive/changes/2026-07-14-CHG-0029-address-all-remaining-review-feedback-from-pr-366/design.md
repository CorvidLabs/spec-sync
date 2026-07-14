---
change: CHG-0029-address-all-remaining-review-feedback-from-pr-366
artifact: design
---

# Design

Keep the fixes narrow and reuse existing authorities:

- Track whether the caller supplied delivery paths before generated ledger coverage is appended.
- Resolve canonical spec and companion paths through the existing safe registry helper when evaluating coverage and acceptance input membership.
- Add the registry path to default protected meaningful inputs.
- Interpret Cargo target-selection flags and the `--` binary-argument boundary before classifying a nested command as SpecSync.
- Decide whether SDD is enabled before validating lifecycle-owned state.
- Apply the existing inherited verification diagnostic to all three lifecycle command families before dispatch.
