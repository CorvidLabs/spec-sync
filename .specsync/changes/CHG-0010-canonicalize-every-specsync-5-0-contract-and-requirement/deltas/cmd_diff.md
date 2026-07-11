## ADDED

### REQUIREMENT REQ-cmd-diff-001

The diff command SHALL identify affected specs and export drift from a safe Git base comparison in every supported output format.

Acceptance Criteria
- `cmd_diff` lists every spec whose `files:` frontmatter references a file changed since the base ref, plus specs whose own `.spec.md` file changed.
- For each affected spec, computes `new_exports` (exported symbols not documented in the spec body) and `removed_exports` (spec-documented symbols no longer exported).
- When `base == "HEAD"` and running inside a GitHub Actions `pull_request`/`pull_request_target` event with `GITHUB_BASE_REF` set, compares against `origin/<base_ref>` instead of `HEAD`.
- Supports JSON, Markdown/Github, and Text/Table/Csv output formats; empty changesets render a "no changes" message per format.
- When no specs are affected, the Text format lists changed source files not covered by any spec (filtered by `config.source_extensions`).
