---
title: Architecture
layout: default
nav_order: 8
---

# Architecture
{: .no_toc }

How SpecSync is built. Useful for contributors and anyone adding language support.
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
├── types.rs             Core data types, config schema, enums
├── config.rs            specsync.json / specsync.toml loading + auto-detection
├── parser.rs            Frontmatter + spec body parsing
├── validator.rs         Validation pipeline + coverage computation
├── generator.rs         Spec scaffolding (template + AI-powered)
├── ai.rs                AI provider resolution, prompt building, API/CLI execution
├── scoring.rs           Spec quality scoring (0–100, weighted rubric)
├── mcp.rs               MCP server (JSON-RPC over stdio, tools for check/generate/score)
├── watch.rs             File watcher (notify, 500ms debounce)
├── hash_cache.rs        Content-hash cache for incremental validation
├── registry.rs          Cross-project module registry (specsync-registry.toml)
├── manifest.rs          Package manifest parsing (package.json, Cargo.toml, go.mod, etc.)
├── schema.rs            SQL schema parsing for db_tables validation
├── merge.rs             Git conflict resolution for spec files
├── archive.rs           Task archival from companion tasks.md files
├── compact.rs           Changelog compaction (trim old entries)
├── view.rs              Role-filtered spec viewing (dev, qa, product, agent)
├── github.rs            GitHub integration (repo detection, drift issues)
└── exports/
    ├── mod.rs            Language dispatch + file utilities
    ├── typescript.rs     TS/JS exports
    ├── rust_lang.rs      Rust pub items
    ├── go.rs             Go uppercase identifiers
    ├── python.rs         Python __all__ / top-level
    ├── swift.rs          Swift public/open items
    ├── kotlin.rs         Kotlin top-level
    ├── java.rs           Java public items
    ├── csharp.rs         C# public items
    ├── dart.rs           Dart public items
    ├── php.rs            PHP public classes/functions
    └── ruby.rs           Ruby public methods/classes
```

---

## Design Principles

**Single binary, no runtime deps.** Download and run. No Node.js, no Python, no package managers.

**Zero YAML dependencies.** Frontmatter parsed with a purpose-built regex parser. Keeps the binary small and compile times fast.

**Regex-based export extraction.** Each language backend uses pattern matching, not AST parsing. Trades some precision for portability — works without compilers or language servers installed.

**Release-optimized.** LTO, symbol stripping, `opt-level = 3`.

---

## Validation Pipeline

### Stage 1: Structural

- Parse YAML frontmatter
- Check required fields: `module`, `version`, `status`, `files`
- Verify every file in `files` exists on disk
- Check all `requiredSections` present as `## Heading` lines
- Validate `depends_on` paths exist
- Validate `db_tables` exist in schema files (if `schemaDir` configured)

### Stage 2: API Surface

- Detect language from file extensions
- Extract public exports using language-specific regex
- Extract symbol names from Public API tables (backtick-quoted)
- **In spec but not in code** = Error (phantom/stale)
- **In code but not in spec** = Warning (undocumented)

### Stage 3: Dependencies

- `depends_on` paths must point to existing spec files
- `### Consumed By` section: referenced files must exist

---

## Adding a Language

1. **Create extractor** — `src/exports/yourlang.rs`, return `Vec<String>` of exported names
2. **Add enum variant** — `Language` in `src/types.rs`
3. **Wire dispatch** — in `src/exports/mod.rs`: extension detection, match arm, test file patterns
4. **Write tests** — common patterns, edge cases, test file exclusion

Each extractor: strip comments, apply regex, return symbol names. No compiler needed.

---

## Dependencies

| Crate | Purpose |
|:------|:--------|
| `clap` | CLI parsing (derive macros) |
| `serde` + `serde_json` | JSON for config and `--json` output |
| `regex` | Export extraction + frontmatter parsing |
| `walkdir` | Recursive directory traversal |
| `colored` | Terminal colors |
| `notify` + `notify-debouncer-full` | File watching for `watch` command |
| `ureq` | HTTP client for Anthropic/OpenAI API calls |
| `sha2` | Content hashing for incremental validation cache |

### Dev

| Crate | Purpose |
|:------|:--------|
| `tempfile` | Temp dirs for integration tests |
| `assert_cmd` | CLI test utilities |
| `predicates` | Output assertions |
