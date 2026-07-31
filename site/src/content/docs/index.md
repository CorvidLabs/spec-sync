---
title: "SpecSync"
section: "Getting started"
order: 0
---

Verified spec-driven development and bidirectional spec-to-code validation. Written in Rust. Single binary. 33 languages.

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
specsync change new "Add auth"  # start the deterministic SDD interview
specsync change answer CHG-... acceptance_criteria "Auth succeeds" --json
specsync change approve CHG-... # one human scope approval
# implement code, specs, and tests
specsync change check CHG-...   # scoped verify for this change (not archive history)
specsync change audit           # active workspaces + living specs
# open/update the PR for ordinary + scoped review
specsync change finalize CHG-... # same-PR metadata/archive-only finalization
# GitHub performs the merge
specsync check                  # validate specs against code
specsync coverage               # see what's covered
specsync generate               # scaffold specs for unspecced modules
specsync agents install                     # install native coding-agent workflows
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

All 33 extractors are auto-detected from file extensions with no per-language configuration:

TypeScript/JavaScript, Rust, Go, Python, Swift, Kotlin, Java, C#, Dart, PHP, Ruby, YAML, C, C++, Scala, Crystal, Nim, Erlang, Elixir, Perl, Lisp, Haskell, Lua, R, OCaml, Groovy, F#, Clojure, D, Objective-C, Bash, PowerShell, and Vala.

## Learn More

| New to SpecSync? | Already using it? |
|:-----------------|:-----------------|
| [Quick Start Guide](quickstart.md) — up and running in 5 min | [CLI Reference](cli.md) — commands, flags, and output formats |
| [Why SpecSync?](why-specsync.md) — comparison with alternatives | [Configuration](configuration.md) — `.specsync/config.toml` options |
| [Spec Kit comparison](comparisons/spec-kit/) — lifecycle and artifact differences | [OpenSpec comparison](comparisons/openspec/) — deltas, archives, and enforcement |
| [Use them together](comparisons/using-together/) — artifact equivalence and integration | [Adversarial proof](comparisons/adversarial-proof/) — what each core actually detects |
| [Spec Format](spec-format.md) — how to write specs | [Cross-Project Refs](cross-project-refs.md) — multi-repo validation |
| [Companion Files](companion-files.md) — requirements, tasks, context, and evidence | [Language Reference](/spec-sync/languages/) — extraction rules and caveats |
| [Workflow Guide](workflow.md) — full lifecycle | [AI Agents](integrations/ai-agents.md) — native skills + MCP |
| [Architecture](architecture.md) — how it works | [VS Code Extension](integrations/vscode-extension.md) — editor integration |
