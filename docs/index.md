---
title: Home
layout: home
nav_order: 1
---

# SpecSync
{: .fs-9 }

Bidirectional spec-to-code validation. Written in Rust. Single binary. 11 languages. VS Code extension.
{: .fs-6 .fw-300 }

[Get Started](#quick-start){: .btn .btn-primary .fs-5 .mb-4 .mb-md-0 .mr-2 }
[View on GitHub](https://github.com/CorvidLabs/spec-sync){: .btn .fs-5 .mb-4 .mb-md-0 }

---

## The Problem

Specs reference functions that were renamed. Code exports things the spec doesn't mention. Nobody notices until someone reads the docs and gets confused. SpecSync catches this automatically by validating `*.spec.md` files against actual source code — in both directions.

| Direction | Severity |
|:----------|:---------|
| Code exports something not in the spec | Warning |
| Spec documents something missing from code | **Error** |
| Source file in spec was deleted | **Error** |
| DB table in spec missing from schema | **Error** |
| Column in spec missing from migrations | **Error** |
| Column in schema not documented in spec | Warning |
| Column type mismatch between spec and schema | Warning |
| Required section missing | **Error** |

---

## Quick Start

```bash
cargo install specsync          # or use the GitHub Action, or download a binary
specsync init                   # create specsync.json
specsync check                  # validate specs against code
specsync coverage               # see what's covered
specsync generate               # scaffold specs for unspecced modules
specsync generate --provider auto           # AI-powered specs (auto-detect provider)
specsync generate --provider anthropic      # use Anthropic API directly
specsync score                  # quality-score your specs (0–100)
specsync add-spec auth          # scaffold a single spec with companion files
specsync resolve --remote       # verify cross-project spec references
specsync init-registry          # publish your modules for other projects
specsync hooks install          # install agent instructions + git hooks
specsync mcp                    # start MCP server for AI agents
specsync watch                  # re-validate on file changes
```

---

## Supported Languages

Auto-detected from file extensions. No per-language configuration.

TypeScript/JS, Rust, Go, Python, Swift, Kotlin, Java, C#, Dart, PHP, Ruby.

See [Workflow Guide](workflow) for the end-to-end workflow, [Spec Format](spec-format) for how to write specs, [CLI Reference](cli) for all commands, [VS Code Extension](vscode-extension) for editor integration, [Cross-Project References](cross-project-refs) for multi-repo validation, and [Configuration](configuration) for `specsync.json` options.
