---
change: CHG-0022-preserve-canonical-change-log-table-schemas-when-accepting-semantic-deltas
artifact: testing
---

# Testing

Regression coverage exercises the row generator directly for all supported compatibility boundaries:

- `REQ-change-021` is exercised by the three focused `append_changelog_*` tests in `src/change.rs`.
- `Version | Date | Changes` uses the post-bump frontmatter version and produces three aligned cells.
- `Date | Author | Change` records `SpecSync` without shifting the description.
- `Date | Change` retains the established two-column row.

Run `fledge run test -- append_changelog` while implementing. Before verification, run the complete native
`fledge run test`, formatting, type-check, lint, release-build, audit, and strict 100% SpecSync lanes.
