---
title: Architecture
layout: default
nav_order: 7
---

# Architecture
{: .no_toc }

How SpecSync is built — useful if you want to contribute or add language support.
{: .fs-6 .fw-300 }

<details open markdown="block">
  <summary>Table of contents</summary>
  {: .text-delta }
1. TOC
{:toc}
</details>

---

## Source Layout

```
src/
├── main.rs              CLI entry point (clap) + output formatting
├── types.rs             Core data types + config schema
├── config.rs            specsync.json loading
├── parser.rs            YAML frontmatter + spec body parsing
├── validator.rs         Spec validation + coverage computation
├── generator.rs         Spec scaffolding for new modules
├── watch.rs             File watcher (notify crate, 500ms debounce)
└── exports/
    ├── mod.rs            Language dispatch + file utilities
    ├── typescript.rs     TypeScript/JS export extraction
    ├── rust_lang.rs      Rust pub item extraction
    ├── go.rs             Go exported identifier extraction
    ├── python.rs         Python __all__ / top-level extraction
    ├── swift.rs          Swift public/open item extraction
    ├── kotlin.rs         Kotlin public item extraction
    ├── java.rs           Java public item extraction
    ├── csharp.rs         C# public item extraction
    └── dart.rs           Dart public item extraction
```

---

## Design Principles

### Single binary, no runtime dependencies

SpecSync ships as one static binary. No Node.js, no Python, no package managers. Download it and run it.

### Zero YAML dependencies

Frontmatter is parsed with a purpose-built regex parser — no YAML library in the dependency tree. This keeps the binary small and compile times fast.

### Language-agnostic export extraction

Each language backend lives in `src/exports/` and implements export detection via regex pattern matching. No AST parsing, no language-specific toolchains required. This trades some precision for massive portability — SpecSync works anywhere without needing compilers or language servers installed.

### Release-optimized builds

The release profile uses LTO (Link-Time Optimization), symbol stripping, and `opt-level = 3` for maximum performance and minimum binary size.

---

## Validation Pipeline

Validation runs in three stages:

### Stage 1: Structural Validation

- Parse YAML frontmatter from the spec file
- Check required fields: `module`, `version`, `status`, `files`
- Verify every file in the `files` list exists on disk
- Check that all `requiredSections` are present as `## Heading` lines
- Validate `depends_on` spec paths exist
- Validate `db_tables` exist in schema files (if `schemaDir` is configured)

### Stage 2: API Surface Validation

- Detect language from file extensions
- Extract public exports from each source file using language-specific regex
- Extract symbol names from Public API tables in the spec (backtick-quoted)
- Compare the two sets:
  - **Symbol in spec but not in code** = Error (phantom/stale documentation)
  - **Symbol in code but not in spec** = Warning (undocumented export)

### Stage 3: Dependency Validation

- Check `depends_on` paths point to existing spec files
- If a `### Consumed By` section exists, validate referenced files exist

---

## Adding a New Language

To add support for a new language:

1. **Create the extractor** — add `src/exports/yourlang.rs` with a function that takes file contents and returns a `Vec<String>` of exported symbol names

2. **Add the Language variant** — in `src/types.rs`, add your language to the `Language` enum

3. **Wire up dispatch** — in `src/exports/mod.rs`:
   - Add file extension detection in the language detection function
   - Add a match arm to call your extractor
   - Add test file patterns for your language

4. **Write tests** — cover common export patterns, edge cases, and test file exclusion

### Example: What an extractor looks like

Each extractor follows the same pattern:
- Strip comments from the source code
- Apply regex patterns to find public/exported declarations
- Return symbol names as strings

The regex approach means you don't need the language's compiler or parser installed — just pattern matching on source text.

---

## Dependencies

| Crate | Purpose |
|:------|:--------|
| `clap` | CLI argument parsing with derive macros |
| `serde` + `serde_json` | JSON serialization for config and `--json` output |
| `regex` | Pattern matching for export extraction and frontmatter parsing |
| `walkdir` | Recursive directory traversal |
| `colored` | Colored terminal output |
| `notify` + `notify-debouncer-full` | File system watching for `watch` command |

### Dev dependencies

| Crate | Purpose |
|:------|:--------|
| `tempfile` | Temporary directories for integration tests |
| `assert_cmd` | CLI testing utilities |
| `predicates` | Assertions for CLI output matching |
