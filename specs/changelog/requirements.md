---
spec: changelog.spec.md
---

## User Stories

- As a release manager, I want a structured diff of spec changes between two git refs so that I can summarize what changed in a release without reading every spec by hand
- As a reviewer, I want field-level changes (status, version, files, deps) called out per spec so that I can spot risky metadata drift during review
- As a CI operator, I want changelog output in text, JSON, and markdown so that I can render it in terminals, parse it programmatically, or paste it into release notes
- As a developer, I want the changelog computed from git history without touching my working tree so that running it never disturbs uncommitted work
- As a tooling author, I want a `parse_range` helper that validates `from..to` ranges so that malformed ranges are rejected before any git calls

## Acceptance Criteria

- `generate_changelog(root, specs_dir, from_ref, to_ref)` returns a `ChangelogReport` with `added`, `removed`, and `modified` lists
- Spec state at each ref is read via `git ls-tree` and `git show` — the working tree and index are never modified
- Only files ending in `.spec.md` under the specs dir are considered
- A spec appears in `added` if present at `to_ref` but not `from_ref`; in `removed` if present at `from_ref` but not `to_ref`
- A spec appears in `modified` only when its raw content differs AND at least one frontmatter field or body section actually changed
- Frontmatter comparison covers: `status`, `version`, `module`, `files`, `db_tables`, `depends_on`, `agent_policy`, `implements`, `tracks`
- Body comparison emits `section:<name>` changes marked `(added)`, `(removed)`, or `(modified)` for top-level `##` sections
- `parse_range` splits on the first `..`, requires both sides non-empty, and returns `None` otherwise (e.g. `v0.1`, `..v0.2`, `v0.1..` all yield `None`)
- `format_text`, `format_json`, and `format_markdown` all consume the same `ChangelogReport` and report identical add/change/remove counts
- Empty reports render a "No spec changes detected" message in text and markdown
- Output is deterministic: added/removed/modified lists are sorted by module name

## Constraints

- Must use `parse_frontmatter` (parser module) and the `Frontmatter` type (types module) — no bespoke YAML parsing
- Must not modify the working tree or index; all reads go through `git ls-tree`/`git show`
- `\r\n` line endings in git-read content are normalized to `\n` before parsing
- A missing or nonexistent git ref yields an empty spec list rather than an error or panic
- Specs whose frontmatter fails to parse at a ref are silently skipped, not reported as changed
- Output for the same two refs must be deterministic (sorted by module)

## Out of Scope

- Generating changelogs for source code (non-spec) files
- Writing the changelog to a CHANGELOG.md file or committing it (caller's responsibility)
- Diffing line-by-line within a section — section changes are reported only as added/removed/modified
- Semantic versioning inference or release-tag automation
- Comparing more than two refs at once or producing a cumulative multi-release log

### REQ-changelog-001

The `changelog` module SHALL preserve truthful user-visible behavior for the pre-6.0 product fixes landed in this change.

Acceptance Criteria
- Related tests remain green.
- No intentional regression of SpecSync 6.0 lifecycle verbs.

