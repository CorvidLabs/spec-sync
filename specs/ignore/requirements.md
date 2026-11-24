---
spec: ignore.spec.md
---

## User Stories

- As a developer, I want to suppress specific warning categories globally via a `.specsyncignore` file so that intentional gaps don't create noise on every run
- As a developer, I want to suppress a category for only a subtree of specs (e.g. `stub-section:specs/legacy/`) so that legacy specs don't block new work while keeping checks strict elsewhere
- As a spec author, I want to silence a warning for a single spec with an inline `<!-- specsync-ignore: ... -->` comment so that the suppression lives next to the spec it applies to
- As a developer, I want category names to accept aliases and kebab/snake case so that the ignore syntax is forgiving

## Acceptance Criteria

- `WarningCategory::from_str` parses category names case-insensitively, treating `_` and `-` as equivalent, and accepts short aliases (`requirements`, `stub`, `undocumented`, `schema-mismatch`, `changelog`, `invariants`, `depends-on`); unknown strings return `None`
- `WarningCategory::classify` maps a warning's text to a category, checking the more specific `schema-type-mismatch` before the generic `schema-column`
- `IgnoreRules::load` reads `.specsyncignore` from the project root, skipping blank lines and `#` comments and stripping inline `#` comments; a `category:pattern` line becomes a per-spec rule, a bare `category` becomes a global rule; a missing file yields empty (default) rules, not an error
- `IgnoreRules::parse_inline` extracts categories from `<!-- specsync-ignore: a, b -->` lines in a spec body; a directive missing the closing `-->` is ignored
- `is_suppressed` returns true when the warning's category is suppressed globally, inline, or by a per-spec pattern that the spec's relative path `starts_with`; a warning whose text matches no category (`classify` → `None`) is never suppressed

## Constraints

- Pure in-memory rule evaluation — no process exit; only `load` touches the filesystem (read-only)
- Per-spec patterns match by path prefix (`starts_with`), not glob
- Adding a new suppressible warning requires a `WarningCategory` variant plus matching arms in both `from_str` and `classify`

## Out of Scope

- Glob/regex matching for per-spec patterns (prefix matching only)
- Suppressing errors (only warnings are categorized and suppressible)
- Configuring ignore rules through `specsync.json` (only `.specsyncignore` and inline comments)

### REQ-ignore-001

The `ignore` module SHALL preserve truthful user-visible behavior for the pre-6.0 product fixes landed in this change.

Acceptance Criteria
- Related tests remain green.
- No intentional regression of SpecSync 6.0 lifecycle verbs.

### REQ-ignore-002

Ignore rules SHALL read `.specsyncignore` only through the bounded project read of `util` (REQ-util-003), with a bound of 4 MiB. A read refused there, because the file leaves the project through a symbolic link, is a dangling link, is not a regular file, or is larger than the bound, SHALL be handled as that read's other read errors are, with a message that names the path and the reason, and SHALL never wait on the file or read past the bound.

Acceptance Criteria
- With `.specsyncignore` a FIFO, `check` finishes without waiting on it.
- `tests/integration/project_reads.rs` `no_project_file_is_read_outside_the_bounded_read` finds no direct file read in this module's non-test source outside the allowed list

