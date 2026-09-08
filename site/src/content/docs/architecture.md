---
title: "Architecture"
section: "Reference"
order: 4
---

How SpecSync keeps its validation and SDD lifecycle deterministic, portable, and usable by both humans and coding agents.

## System shape

SpecSync is a single Rust binary with five cooperating layers:

```text
CLI / MCP / agent skills
        ↓
SDD change lifecycle and canonical spec model
        ↓
structural, API, dependency, and schema validation
        ↓
language-specific regex or tree-sitter extraction
        ↓
Git, filesystem, registry, and GitHub adapters
```

The CLI is the source of truth. Editor extensions, GitHub Actions, MCP clients, and native agent skills invoke the same commands and receive the same results.

## Source layout

| Area | Responsibility |
|---|---|
| `main.rs`, `cli.rs`, `cli_args.rs` | Command parsing, dispatch, enforcement, and output formats |
| `change.rs` | Verified SDD interviews, approvals, deltas, evidence, acceptance, and archival |
| `parser.rs`, `validator.rs` | Spec structure, Public API tables, coverage, and canonical checks |
| `deps.rs` | Source import discovery and dependency graph validation |
| `exports/` | Language detection and public-symbol extraction |
| `schema.rs` | SQL table and column drift validation |
| `config.rs`, `types.rs` | Canonical TOML configuration and shared data models |
| `generator.rs`, `commands/` | Scaffolding and command implementations |
| `hash_cache.rs`, `watch.rs` | Incremental validation and filesystem watching |
| `registry.rs`, `importer.rs` | Cross-project references and external spec import |
| `github.rs`, `comment.rs` | GitHub metadata, drift issues, and PR summaries |
| `agents.rs`, `hooks.rs`, `mcp.rs` | Coding-agent skills, Git hooks, and JSON-RPC integration |

## Design principles

**Deterministic local core.** Spec checking, generation, scoring, dependency analysis, and lifecycle enforcement do not call a model or require an API key.

**Hybrid extraction.** Regex extraction is portable and remains the default. Tree-sitter AST extraction is available for supported languages when `parse_mode = "ast"` is configured. Per-language reference pages document the exact behavior and caveats.

**Fail-closed evidence.** Scope approval, verification, scoped review, and finalization are
digest-bound. Changes to scoped code, tests, configuration, file kind, executable mode, or
contract inputs invalidate stale evidence. Strict policy adds validators to the same evidence path.

**Git-native history.** Canonical specs, semantic deltas, requirements, decisions, and evidence remain reviewable as ordinary files. New changes archive on the same PR before GitHub merges the delivery.

**One engine for humans and agents.** Native skills present the deterministic CLI interview conversationally; they do not create a separate agent-only lifecycle.

## Validation pipeline

Ordinary `specsync check` performs structural validation; it does not audit active change workspaces or archives, even when SDD is enabled.

A successful check validates configured structure, exported API names, source mappings, dependency declarations, and supported schema rules. It does not prove that arbitrary natural-language requirements describe the implementation; reviewers and product tests establish those behaviors.

### Optional lifecycle validation

Use `change check` for scoped evidence and spec-to-code validation, and `change audit` for active-workspace and living-spec health. Lifecycle commands enforce applicable approval, semantic delta, task, evidence, and path-coverage policy. Product tests run separately in CI.

Approval digests bind recorded approval to content. The actor/reviewer label is not authenticated identity. Signed provenance and a required policy-verification check must be configured separately when identity or provenance enforcement is required; recording a signature or using soft mode alone is not that gate.

### 1. Structural contract

- Parse frontmatter and required fields
- Verify source files and dependency specs exist
- Require configured `##` sections
- Validate requirement and companion conventions

### 2. API and schema surface

- Detect each source language
- Extract public symbols using its configured parser
- Compare symbols with backtick-quoted Public API entries
- Compare documented database tables and columns with migrations

Spec-only symbols are errors. Code-only symbols and undocumented schema details are warnings that become failures in strict mode.

### 3. Dependency graph

Source imports are mapped to owning specs and compared with `depends_on`. Strict mode rejects undeclared imports, missing modules, and cycles.

### 4. Coverage and quality

Coverage measures source files and LOC governed by specs. Scoring evaluates frontmatter, required sections, API completeness, depth, and freshness.

## Adding a language

1. Add or update the extractor in `src/exports/`.
2. Register extensions, test exclusions, and parser support.
3. Add focused fixtures for visibility, comments, strings, re-exports, and malformed input.
4. Update the language registry data in `site/src/data/languages/`.
5. Run the full export verification suite and strict SpecSync check.

Extractors must fail safely on unreadable input and must not require a language compiler or server at runtime.

## Important dependencies

| Crate | Purpose |
|---|---|
| `clap` | CLI parsing |
| `serde`, `serde_json` | Configuration, policy, evidence, and JSON output |
| `regex` | Portable parsing and extraction |
| `tree-sitter-*` | Optional AST-backed language extraction |
| `walkdir` | Project discovery and coverage |
| `sha2` | Cache and lifecycle evidence digests |
| `notify` | Watch mode |
| `ureq` | Registry, import, and GitHub HTTP access—not AI inference |

See the [spec format](spec-format.md), [workflow](workflow.md), and [language reference](/spec-sync/languages/) for the user-facing contracts implemented by these layers.
