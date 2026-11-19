---
module: util
version: 4
status: stable
files:
  - src/util.rs
db_tables: []
depends_on: []
---

# Util

## Purpose

Provides small shared utility functions used by validators, fix suggestions, and rule compilation. The module keeps low-level helpers centralized so parsing and validation modules do not duplicate edit-distance or regex-safety logic.

## Public API

### Exported Functions

| Function | Parameters | Returns | Description |
|----------|------------|---------|-------------|
| `levenshtein` | `a: &str, b: &str` | `usize` | Computes Unicode-scalar edit distance between two strings |
| `safe_regex` | `pattern: &str` | `Option<regex::Regex>` | Compiles a user-provided regex with bounded regex and DFA size limits |
| `confine_path_to_root` | `root, rel` | `Option<PathBuf>` | Resolve a project-relative path and reject escapes outside the root |
| `read_project_file` | `root, path, max_bytes` | `io::Result<Option<Vec<u8>>>` | Read a file beneath the project root, `None` when nothing is there; follows a link that stays inside the root and refuses one that leaves it, a dangling link, a non-regular file and a file over `max_bytes`, opening without blocking and reading at most one byte past the bound (REQ-util-002, REQ-util-003) |
| `read_project_bytes` | `root, path, max_bytes` | `io::Result<Vec<u8>>` | `read_project_file` with the contract of `std::fs::read`: a missing file is a not-found error (REQ-util-003) |
| `read_project_string` | `root, path, max_bytes` | `io::Result<String>` | `read_project_file` with the contract of `std::fs::read_to_string`: a missing file is not found, bytes that are not UTF-8 are invalid data (REQ-util-003) |
| `MAX_STATE_FILE_BYTES` | constant | `u64` | 4 MiB: the bound for policy, config and state files and change records |
| `MAX_SPEC_FILE_BYTES` | constant | `u64` | 8 MiB: the bound for spec files, companions, history fragments, templates and hook files |
| `MAX_SOURCE_FILE_BYTES` | constant | `u64` | 8 MiB: the bound for source and schema files |
| `MAX_PROJECT_INPUT_BYTES` | constant | `u64` | 64 MiB: the bound for the transaction journal, the hash cache and other whole project inputs |
| `write_project_file` | `root, path, contents` | `io::Result<()>` | Write a file beneath the project root, creating it and its directories when missing; writes through a link that stays inside, refuses the rest and never creates through a link |
| `create_project_file` | `root, path, contents` | `io::Result<bool>` | Create a file that does not exist yet with an exclusive create; `false` and no change when a regular file is already there |
| `remove_project_file` | `root, path` | `io::Result<()>` | Remove a name beneath the project root; a link that stays inside is removed as a link |
| `create_project_dir` | `root, path` | `io::Result<()>` | Create a directory beneath the project root and the directories on the way |
| `project_path_refusal` | `root, path` | `Option<String>` | Why the project file functions refuse a path, naming it, without changing anything; `None` for a path they accept |

## Invariants

1. `levenshtein` treats input as `char` sequences rather than raw bytes.
2. `levenshtein` returns `0` for equal strings and the other string's character length when one side is empty.
3. `safe_regex` never panics on invalid user input; invalid or oversized patterns return `None`.
4. `safe_regex` applies the same maximum limit to compiled regex size and DFA size.
5. `read_project_file`, `write_project_file`, `create_project_file`, `remove_project_file` and `create_project_dir` act only through a `cap_std` directory handle on the project root, so nothing they touch is outside it, even if a path component is swapped while they run. A symbolic link that stays inside the root is followed. A path that leaves the root through a link (an absolute target included) or through `..`, a dangling link, and a target of the wrong kind are refused with an error naming the path. A missing file is created with an exclusive create, never through a link (REQ-util-002).
6. `read_project_file`, `read_project_bytes` and `read_project_string` open the file through the project root handle without blocking, check on the open handle that it is a regular file, refuse it when it is larger than the bound the caller passes, and read at most one byte past that bound. A FIFO or a device therefore never holds a read, and no read grows without limit. The bounds are named constants: 4 MiB for state, 8 MiB for specs and source, 64 MiB for whole project inputs (REQ-util-003).

## Behavioral Examples

### Scenario: Suggest nearby filenames

- **Given** the strings `config.ts` and `confg.ts`
- **When** `levenshtein` compares them
- **Then** it returns `1`, allowing validation to suggest the near miss

### Scenario: Reject invalid regex

- **Given** an invalid pattern such as `[invalid`
- **When** `safe_regex` tries to compile it
- **Then** it returns `None` instead of propagating a regex parser error

## Error Cases

| Condition | Behavior |
|-----------|----------|
| Empty string passed to `levenshtein` | Returns the character length of the other string |
| Invalid regex syntax | `safe_regex` returns `None` |
| Pattern exceeds configured regex size limits | `safe_regex` returns `None` |
| A project path leaves the root through a symbolic link or `..`, is a dangling link, or is not a regular file | The project file functions return an error naming the path and change nothing; `project_path_refusal` returns the same reason |
| A project file is a FIFO, a device, or larger than the bound passed | The read functions return an error naming the path and the reason (`is not a regular file`, `is larger than <n> bytes`) without waiting or reading past the bound |

## Dependencies

### Consumes

| Module | What is used |
|--------|--------------|
| regex | `RegexBuilder` for bounded regex compilation |

### Consumed By

| Module | What is used |
|--------|--------------|
| validator | `levenshtein`, `safe_regex` |
| parser | `levenshtein` |

## Change Log

| Date | Change |
|------|--------|
| 2026-10-10 | Add `util.3md` beside the spec. The first plane is context and later planes are lessons. `context.md` stays. |
| 2026-06-07 | Initial spec for shared utility helpers |
| 2026-07-11 | CHG-0010-canonicalize-every-specsync-5-0-contract-and-requirement: Canonicalize every SpecSync 5.0 contract and requirement |
| 2026-08-01 | CHG-0071-land-pre-6-0-product-fixes-for-hooks-init-coverage-naming-and-exit-codes-scoped: Land pre-6.0 product fixes for hooks init coverage naming and exit codes (scoped paths) |
