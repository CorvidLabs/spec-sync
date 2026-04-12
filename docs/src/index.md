# SpecSync

Bidirectional spec-to-code validation. Written in Rust. Single binary. 11 languages. VS Code extension.

[Get Started](quickstart.md)
[Why SpecSync?](why-specsync.md)
[View on GitHub](https://github.com/CorvidLabs/spec-sync)

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
specsync init                   # create .specsync/config.toml
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

## Learn More

| New to SpecSync? | Already using it? |
|:-----------------|:-----------------|
| [Quick Start Guide](quickstart.md) — up and running in 5 min | [CLI Reference](cli.md) — all 14 commands |
| [Why SpecSync?](why-specsync.md) — comparison with alternatives | [Configuration](configuration.md) — `.specsync/config.toml` options |
| [Spec Format](spec-format.md) — how to write specs | [Cross-Project Refs](cross-project-refs.md) — multi-repo validation |
| [Workflow Guide](workflow.md) — full lifecycle | [AI Agents](ai-agents.md) — MCP server + AI generation |
| [Architecture](architecture.md) — how it works | [VS Code Extension](vscode-extension.md) — editor integration |
