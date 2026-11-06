---
change: retire-the-contract-clauses-that-promise-change-check-runs-configured-verification-commands
artifact: docs
---

# Docs

Public pages:

- `site/src/content/docs/why-specsync.md`: the "Configured tests executed by CI gate" and
  "Verification command shell isolation" rows now say that your CI runs the project's tests and that
  verification runs no configured command.
- `site/src/content/docs/comparisons/adversarial-proof.md`: "Make tests fail" now says your CI blocks
  it and that SpecSync runs no configured command.

Already correct and left unchanged: `site/src/content/docs/configuration.md` ("`change check` does not
execute the list"), `MIGRATION.md` item 7, `docs/ADOPTING.md` and README. `CHANGELOG.md` gets one
Unreleased entry. Contract companions are covered in design.md.
