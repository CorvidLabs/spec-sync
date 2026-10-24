## MODIFIED

### REQUIREMENT REQ-cmd-check-015

`check --fix` SHALL insert each new export row immediately after the last row of the selected contiguous Public API table in the matching section or subsection, never after trailing prose, and SHALL append rows at the end of the section only when that section contains no table.

Acceptance Criteria
- With multiple tables, prefer the first whose header contains `Name`; otherwise use the first table. Both row width and insertion offset come from that table.
- A table under a `### Heading`, a `**Bold**` label, or no label followed by prose receives the row inside the table and the prose is preserved after it.
- A `## Public API` section with no table still receives rows at its end, as before.
- `check --strict` passes on the rewritten spec without further edits.

### SPEC SECTION Behavioral Examples

#### Scenario: Incremental check with cache

- **Given** 25 specs, 3 have changed since last check
- **When** `cmd_check` runs without `--force`
- **Then** only 3 specs are re-validated; 22 are skipped via hash cache and any stored findings are replayed

#### Scenario: Warm cache still reports the previous warning

- **Given** a spec whose first `check` reported an undocumented export, and whose files have not changed
- **When** `cmd_check` runs again without `--force`
- **Then** the same warning identity is present in text and JSON, and JSON `specs_checked` is not 0

#### Scenario: Auto-fix undocumented exports

- **Given** spec is missing export `pub fn new_function()`
- **When** `cmd_check` runs with `--fix`
- **Then** the export is appended to the matching Public API table (functions to the functions table, types to the types table) with a generated description prompt and the file is rewritten

#### Scenario: Auto-fix extends an existing table instead of trailing prose

- **Given** a `## Public API` section whose table (under a `### Heading`, a `**Bold**` label, or no label) is followed by prose such as an "Acceptance Criteria" paragraph
- **When** `cmd_check` runs with `--fix` for an undocumented export
- **Then** the new row is inserted directly after the last row of that table, the prose is preserved after it, pipe-shaped lines inside fenced or indented code examples are never treated as table rows, and a subsequent `check --strict` passes; only a section with no table at all receives rows at its end

#### Scenario: Auto-fix chooses one contiguous table (#768)

- **Given** an export subsection or flat Public API with multiple tables
- **When** `check --fix` adds an undocumented export
- **Then** it prefers the first table with a `Name` header (otherwise the first table), derives both width and insertion offset from that same contiguous table, and preserves all other content

#### Scenario: Auto-fix preserves fenced examples while renaming headers

- **Given** a `## Public API` section that contains a fenced markdown example and a near-miss or bare `###` heading
- **When** `cmd_check` runs with `--fix`
- **Then** the heading is normalized and the fenced example body is preserved verbatim

#### Scenario: JSON output format

- **Given** `--format json` is set
- **When** validation completes with errors and warnings
- **Then** output is a single JSON object with `specs_checked`, `passed`, `errors`, `warnings`, `coverage`, and `exit_code` fields
