---
change: check-reports-a-module-s-3md-lesson-file-that-does-not-parse
artifact: design
---

# Design

The check sits with the other companion checks in `validate_spec_content_internal`
(`src/validator/consumed_by_re.rs`), under `validate_companions`, so it runs exactly where the
`requirements.md` duplicate-id check runs and nowhere a confined snapshot validates. It reads
`<spec dir>/<module>.3md` when that file exists, parses it with the same `threemd::parse` the
lesson reader uses, and on failure pushes one warning:
`<path> does not parse as 3md (<error>); its lessons are not read, and <spec dir>/context.md is used
instead`. A file that cannot be read gets the same warning with the read error. Warnings already
fail `--strict`, so nothing else changes.
