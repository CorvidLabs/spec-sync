---
title: Home
layout: home
nav_order: 1
---

# SpecSync
{: .fs-9 }

Bidirectional spec-to-code validation. Written in Rust. Single binary. 9 languages.
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
| Required section missing | **Error** |

---

## Quick Start

```bash
cargo install specsync          # or use the GitHub Action, or download a binary
specsync init                   # create specsync.json
specsync check                  # validate specs against code
specsync coverage               # see what's covered
specsync generate               # scaffold specs for unspecced modules
specsync generate --ai          # AI-powered specs (reads code, writes content)
specsync watch                  # re-validate on file changes
```

---

## Supported Languages

Auto-detected from file extensions. No per-language configuration.

TypeScript/JS, Rust, Go, Python, Swift, Kotlin, Java, C#, Dart.

See [Spec Format](spec-format) for how to write specs, [CLI Reference](cli) for all commands, and [Configuration](configuration) for `specsync.json` options.
