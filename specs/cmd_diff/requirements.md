---
spec: cmd_diff.spec.md
---

## User Stories

- As a developer opening a PR, I want to see which specs are affected by my source changes so that I update the right specs before review.
- As a reviewer, I want export deltas (new/removed symbols vs. the spec) per affected spec so that I can spot undocumented or stale API surface.
- As a CI operator, I want the diff to auto-detect the PR base branch and emit Markdown/JSON so that I can post drift reports without configuring a base ref by hand.

## Acceptance Criteria

- `cmd_diff` lists every spec whose `files:` frontmatter references a file changed since the base ref, plus specs whose own `.spec.md` file changed.
- For each affected spec, computes `new_exports` (exported symbols not documented in the spec body) and `removed_exports` (spec-documented symbols no longer exported).
- When `base == "HEAD"` and running inside a GitHub Actions `pull_request`/`pull_request_target` event with `GITHUB_BASE_REF` set, compares against `origin/<base_ref>` instead of `HEAD`.
- Supports JSON, Markdown/Github, and Text/Table/Csv output formats; empty changesets render a "no changes" message per format.
- When no specs are affected, the Text format lists changed source files not covered by any spec (filtered by `config.source_extensions`).

## Constraints

- Invokes `git diff --name-only --end-of-options <base>`; `--end-of-options` guards against a base ref that begins with `-` being parsed as a git flag (argument injection).
- Must not panic on missing/unreadable spec files or unparseable frontmatter — such specs are skipped.
- Exits with code 1 only when the `git` process itself fails to spawn.

## Out of Scope

- Computing line-level diffs or rendering patch hunks (only changed-file names and export deltas).
- Fetching exports from the base revision via `git show` (deltas compare current exports against spec-documented symbols, not against the base tree).
- Interactive prompts, GUI, or web output.
