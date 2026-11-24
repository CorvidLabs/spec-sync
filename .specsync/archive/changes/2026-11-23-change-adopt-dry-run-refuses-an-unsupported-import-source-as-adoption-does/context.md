---
change: change-adopt-dry-run-refuses-an-unsupported-import-source-as-adoption-does
artifact: context
---

# Context

#432 (filed against 5.2.0). Its main defect, adopt turning on path coverage in a local-only
repository, was confirmed fixed on 2026-11-10. The remaining gap: `adopt`
(`src/change/historical_definition_digest_matches.rs`) returned the dry-run plan before it
called `validate_foreign_import`, so `change adopt --dry-run --source bogus` exited 0 and printed
"import active and canonical artifacts from bogus", while `change adopt --source bogus` refused
with `unsupported adoption source`. The help documents `--dry-run` as a preview of adoption and
`--source` as `openspec` or `speckit`.

The fix runs `validate_foreign_import` in the dry-run branch before returning the plan. It only
reads metadata (symlink and file-type checks), so the dry run still writes nothing. The real run
keeps calling it under the project lock, unchanged.

Chough: spec-sync's `.chough/` is still the 0.5.x layout and the installed Chough 0.6.0 refuses
it; migrating it is its own PR and needs Leif's consent (raised with t370). This change runs the
SpecSync lifecycle only, as #1059 and #1062 did.
