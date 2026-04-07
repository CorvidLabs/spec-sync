---
title: Why SpecSync?
layout: default
nav_order: 2
---

# Why SpecSync?
{: .fs-9 }

How SpecSync compares to other documentation validation approaches.
{: .fs-6 .fw-300 }

---

## The Documentation Problem

Every team has experienced it: someone renames a function, the docs still reference the old name, and nobody notices for months. Or worse — an AI agent reads your stale docs and generates code against an API that no longer exists.

Traditional documentation tools fall into two camps:

1. **Auto-generated docs** (JSDoc, rustdoc, Godoc) — accurate but shallow. They tell you _what_ exists, not _why_ it exists or how pieces fit together.
2. **Hand-written docs** (Notion, Confluence, markdown) — rich context but drift from reality within days of being written.

SpecSync occupies a third space: **validated hand-written specs**. You write the spec, SpecSync ensures it stays true.

---

## Comparison Matrix

| Feature | SpecSync | OpenAPI / Swagger | TypeDoc / JSDoc | ADRs | Notion / Confluence |
|:--------|:--------:|:-----------------:|:---------------:|:----:|:-------------------:|
| Validates against source code | **Yes** | Partial (runtime) | No | No | No |
| Catches renamed exports | **Yes** | No | No | No | No |
| Schema/DB drift detection | **Yes** | No | No | No | No |
| Cross-project references | **Yes** | Via `$ref` | No | No | No |
| Works with any language | **11 languages** | Language-specific | JS/TS only | N/A | N/A |
| AI agent integration (MCP) | **Yes** | Via plugins | No | No | No |
| CI/CD integration | **GitHub Action** | Various | Various | Manual | No |
| Spec lifecycle management | **Yes** | No | No | No | Manual |
| Quality scoring | **Yes** | No | No | No | No |
| Single binary, zero deps | **Yes** | Varies | Node.js | N/A | SaaS |
| Git merge conflict resolution | **Yes** | No | No | No | N/A |

---

## Detailed Comparisons

### vs. OpenAPI / Swagger

OpenAPI is excellent for HTTP APIs — it defines request/response schemas and can generate client SDKs. But it only covers the API boundary. It doesn't validate that your internal modules match their documentation, doesn't catch renamed helper functions, and doesn't track database schema drift.

**Use OpenAPI when:** You need to define and document REST/GraphQL APIs for external consumers.

**Use SpecSync when:** You need to ensure internal module documentation stays accurate across your entire codebase — not just the API surface.

**Use both when:** You have a large project with both public APIs and complex internal architecture.

### vs. TypeDoc / JSDoc / rustdoc

Auto-generated documentation tools extract comments from source code. They're always accurate (by definition), but they only document _what's there_ — not architectural decisions, invariants, behavioral constraints, or cross-module relationships.

**Use auto-doc tools when:** You want API reference docs generated from code comments.

**Use SpecSync when:** You need specs that capture _why_ something works the way it does, what invariants must hold, and how modules relate to each other — and you want those specs validated against the real code.

### vs. Architecture Decision Records (ADRs)

ADRs document _decisions_ — why you chose Postgres over MongoDB, or why the auth middleware uses JWTs. They're valuable but static: once written, they don't get validated against the codebase.

**Use ADRs when:** You want to record architectural decisions and their rationale.

**Use SpecSync when:** You want living documentation that stays synchronized with your code. SpecSync specs can include ADR-like context in companion files (`context.md`) while the core spec stays validated.

### vs. Notion / Confluence

Wiki-style tools are great for onboarding docs, runbooks, and team knowledge bases. But they have no connection to source code — documentation rot is inevitable.

**Use wikis when:** You need free-form collaborative documentation for processes, onboarding, and team knowledge.

**Use SpecSync when:** You need technical specs that are provably accurate. If it's in the spec, it's in the code.

---

## What Makes SpecSync Different

### Bidirectional Validation

Most tools check in one direction — either "does the code match the docs?" or "do the docs describe the code?" SpecSync checks both:

- **Spec references something missing from code** → Error (your spec is lying)
- **Code exports something not in the spec** → Warning (your spec is incomplete)

### Language Agnostic

One tool, one format, 11 languages. Whether your project is TypeScript, Rust, Go, Python, Swift, Kotlin, Java, C#, Dart, PHP, or Ruby — same `*.spec.md` format, same validation.

### AI-Native

SpecSync was built for the AI-assisted development era:

- **MCP server mode** lets AI agents query your specs, check coverage, and generate new specs in real time
- **AI-powered generation** creates meaningful spec content (not just templates) using Claude, OpenAI, Ollama, or Copilot
- **Structured output** (JSON mode) integrates cleanly with agent workflows
- **AGENTS.md generation** produces instruction files for Claude Code, Cursor, and Copilot

### Spec Lifecycle

Specs aren't static documents — they have a lifecycle:

```
create → validate → iterate → stabilize → maintain → compact → archive
```

SpecSync manages this lifecycle with companion files (requirements, tasks, context), quality scoring, changelog compaction, and task archival.

### Zero Dependencies

SpecSync is a single Rust binary. No Node.js runtime, no Python virtualenv, no Docker container. Download it and run it. Installs via `cargo install specsync`, a GitHub Action, or a VS Code extension.

---

## When NOT to Use SpecSync

SpecSync is not the right tool if:

- **You only need API reference docs** — use auto-doc tools (TypeDoc, rustdoc) instead
- **Your project has < 3 modules** — the overhead isn't worth it for tiny projects
- **Your team doesn't write specs** — SpecSync validates specs, it doesn't replace the need to write them (though AI generation helps bootstrap)
- **You need runtime API contract testing** — use OpenAPI + contract testing tools instead

---

## Getting Started

Ready to try it?

```bash
# Install
cargo install specsync

# Initialize in your project
specsync init

# Generate specs for all modules
specsync generate

# Validate
specsync check
```

Or see the [full workflow guide](workflow.md) for a step-by-step walkthrough.
