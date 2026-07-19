---
change: CHG-0058-cover-the-chg-0057-delivery-scope-for-the-src-main-rs-migrate-subcommand-dispatc
artifact: context
---

# Context

CHG-0057 delivered the `migrate 5.0` implementation, including the `src/main.rs` subcommand
dispatch and the CLI integration coverage in `tests/integration/change.rs`, but its declared
affected paths omitted both files, so delivery-diff coverage reports them as uncovered. The
lifecycle offers no post-acceptance affected-paths correction, so a small bookkeeping change
declares ownership of the two paths. No code or canonical spec content changes.
