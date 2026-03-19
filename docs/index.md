---
title: Home
layout: home
nav_order: 1
---

# SpecSync

**Bidirectional spec-to-code validation — keep your docs honest.**
{: .fs-6 .fw-300 }

Written in Rust. Language-agnostic. Single binary. Zero config to start.
{: .fs-5 .fw-300 }

[Get Started](#install){: .btn .btn-primary .fs-5 .mb-4 .mb-md-0 .mr-2 }
[View on GitHub](https://github.com/CorvidLabs/spec-sync){: .btn .fs-5 .mb-4 .mb-md-0 }

---

## The Problem

Documentation drifts. Engineers add new exports but forget to update the spec. Specs reference functions that got renamed months ago. Nobody notices until a new team member reads the docs and gets confused.

**SpecSync catches this automatically** by validating your markdown specs against actual source code — in both directions:

| Situation | Severity | What Happens |
|:----------|:---------|:-------------|
| Code exports something not in the spec | Warning | Undocumented export flagged |
| Spec documents something that doesn't exist | **Error** | Stale/phantom entry caught |
| Source file referenced in spec was deleted | **Error** | Missing file detected |
| DB table in spec doesn't exist in schema | **Error** | Phantom table reported |
| Required section missing from spec | **Error** | Incomplete spec flagged |

---

## How It Works

```
                    *.spec.md files
                         |
                    [1] Discover
                         |
                    [2] Parse frontmatter
                         |
              +----------+----------+
              |          |          |
         [3] Structure  [4] API   [5] Dependencies
              |          |          |
         - Required    - Detect   - depends_on exists?
           fields        language  - db_tables in schema?
         - File        - Extract  - Consumed By refs?
           exists?       exports
         - Required    - Compare
           sections?     with spec
              |          |          |
              +----------+----------+
                         |
                    [6] Report
                         |
              +----------+----------+
              |          |          |
           Errors    Warnings   Coverage %
```

1. **Discover** all `*.spec.md` files in your specs directory
2. **Parse** YAML frontmatter using a zero-dependency regex parser
3. **Validate structure** — required fields, required sections, file existence
4. **Validate API surface** — auto-detect language, extract exports, cross-reference against spec tables
5. **Validate dependencies** — check `depends_on` specs and `db_tables` in schema
6. **Report** errors, warnings, and coverage metrics

---

## Install

### Pre-built binaries (recommended)

Download from [GitHub Releases](https://github.com/CorvidLabs/spec-sync/releases):

```bash
# macOS (Apple Silicon)
curl -sL https://github.com/CorvidLabs/spec-sync/releases/latest/download/specsync-macos-aarch64.tar.gz | tar xz
sudo mv specsync-macos-aarch64 /usr/local/bin/specsync

# macOS (Intel)
curl -sL https://github.com/CorvidLabs/spec-sync/releases/latest/download/specsync-macos-x86_64.tar.gz | tar xz
sudo mv specsync-macos-x86_64 /usr/local/bin/specsync

# Linux (x86_64)
curl -sL https://github.com/CorvidLabs/spec-sync/releases/latest/download/specsync-linux-x86_64.tar.gz | tar xz
sudo mv specsync-linux-x86_64 /usr/local/bin/specsync

# Linux (aarch64)
curl -sL https://github.com/CorvidLabs/spec-sync/releases/latest/download/specsync-linux-aarch64.tar.gz | tar xz
sudo mv specsync-linux-aarch64 /usr/local/bin/specsync
```

**Windows:** download `specsync-windows-x86_64.exe.zip` from the [releases page](https://github.com/CorvidLabs/spec-sync/releases).

### From crates.io

```bash
cargo install specsync
```

### From source

```bash
cargo install --git https://github.com/CorvidLabs/spec-sync
```

---

## Quick Start

```bash
# 1. Initialize config
specsync init

# 2. Validate all specs
specsync check

# 3. See coverage
specsync coverage

# 4. Auto-generate specs for unspecced modules
specsync generate

# 5. Watch mode — re-validates on file changes
specsync watch
```

---

## Supported Languages

SpecSync auto-detects the language from file extensions. The same spec format works for all of them — no per-language configuration needed.

| Language | What Gets Detected | Test Files Excluded |
|:---------|:-------------------|:--------------------|
| **TypeScript / JavaScript** | `export function`, `export class`, `export type`, `export const`, `export enum`, re-exports | `.test.ts`, `.spec.ts`, `.d.ts` |
| **Rust** | `pub fn`, `pub struct`, `pub enum`, `pub trait`, `pub type`, `pub const`, `pub static`, `pub mod` | Inline `#[cfg(test)]` modules |
| **Go** | Uppercase identifiers: `func`, `type`, `var`, `const`, methods | `_test.go` |
| **Python** | `__all__` list, or top-level `def`/`class` (excluding `_`-prefixed) | `test_*.py`, `*_test.py` |
| **Swift** | `public`/`open` func, class, struct, enum, protocol, typealias, actor, init | `*Tests.swift`, `*Test.swift` |
| **Kotlin** | Top-level declarations (public by default), excludes `private`/`internal`/`protected` | `*Test.kt`, `*Spec.kt` |
| **Java** | `public` class, interface, enum, record, methods, fields | `*Test.java`, `*Tests.java` |
| **C#** | `public` class, struct, interface, enum, record, delegate, methods | `*Test.cs`, `*Tests.cs` |
| **Dart** | Top-level declarations (no `_` prefix), class, mixin, enum, typedef | `*_test.dart` |
