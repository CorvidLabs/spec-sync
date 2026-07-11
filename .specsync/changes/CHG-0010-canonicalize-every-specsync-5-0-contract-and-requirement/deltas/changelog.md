## ADDED

### REQUIREMENT REQ-changelog-001

The changelog module SHALL compare canonical spec state between two Git refs without modifying the working tree and SHALL render deterministic reports.

Acceptance Criteria
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
