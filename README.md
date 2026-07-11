<div align="center">

# SpecSync

[![GitHub Marketplace](https://img.shields.io/badge/Marketplace-SpecSync-blue?logo=github)](https://github.com/marketplace/actions/spec-sync)
[![spec coverage](https://img.shields.io/endpoint?url=https://corvidlabs.github.io/spec-sync/badges/coverage.json)](https://corvidlabs.github.io/spec-sync/)
[![CI](https://github.com/CorvidLabs/spec-sync/actions/workflows/ci.yml/badge.svg)](https://github.com/CorvidLabs/spec-sync/actions/workflows/ci.yml)
[![Crates.io](https://img.shields.io/crates/v/specsync.svg)](https://crates.io/crates/specsync)
[![Downloads](https://img.shields.io/crates/d/specsync.svg)](https://crates.io/crates/specsync)
[![License: MIT](https://img.shields.io/badge/License-MIT-blue.svg)](LICENSE)
**Verified spec-driven development with bidirectional spec-to-code validation.** Requirements, semantic deltas, human approvals, tests, code, and CI stay traceable through one lifecycle. Written in Rust. Single binary. 33 languages (with AST support for 10).

[5-Minute Start](#get-started-in-5-minutes) &bull; [Spec Format](#spec-format) &bull; [CLI](#cli-reference) &bull; [VS Code Extension](#vs-code-extension) &bull; [Cross-Project Refs](#cross-project-references) &bull; [GitHub Action](#github-action) &bull; [Config](#configuration) &bull; [Docs Site](https://corvidlabs.github.io/spec-sync)

</div>

---

## What It Does

SpecSync validates markdown module specs (`*.spec.md`) against your source code in both directions:

| Direction | Severity | Meaning |
|-----------|----------|---------|
| Code exports something not in the spec | Warning | Undocumented export |
| Spec documents something missing from code | **Error** | Stale/phantom entry |
| Source file in spec was deleted | **Error** | Missing file |
| DB table in spec missing from schema | **Error** | Phantom table |
| Column in spec missing from migrations | **Error** | Phantom column |
| Column in schema not documented in spec | Warning | Undocumented column |
| Column type mismatch between spec and schema | Warning | Type drift |
| Required markdown section missing | **Error** | Incomplete spec |

## Full SDD Lifecycle (5.0)

SpecSync 5.0 manages delivery work as versioned change workspaces:

```text
draft → approved → implementing → verifying → accepted → archived
```

```bash
SPECSYNC_BIN=specsync examples/sdd-lifecycle/run.sh
```

The executable example creates a disposable Git repository and performs every required step: it answers the complete deterministic interview, fills the selected artifacts, records definition approval, implements and commits the change, streams verification, records closing approval, and runs the unified check. For a contract-changing feature, the same gates additionally require a complete semantic delta with module-scoped requirement IDs before definition approval.

```bash
# merge the delivery branch, then archive when its diff no longer needs coverage
specsync change archive CHG-0001-add-passkey-authentication
```

The first and closing approvals are mandatory, portable, digest-bound human gates. During implementation, `specsync check` validates code against canonical specs plus approved semantic deltas. Acceptance requires fresh tests and requirement evidence bound to the tested commit and working-tree inputs, atomically updates canonical requirements/specs, increments spec versions, and records the change ID. Dirty edits after verification invalidate the evidence. Archiving is intentionally post-merge: SpecSync keeps an accepted workspace active while the current delivery diff still depends on its path coverage.

Run the repository's executable [single-change lifecycle](examples/sdd-lifecycle/) and [ordered concurrent changes](examples/sdd-concurrent-changes/) examples to exercise both workflows in disposable projects.

Native Claude, Cursor, Codex, and Gemini integrations conduct the same deterministic interview through JSON commands, so humans and agents are equal clients of one workflow engine.

## Supported Languages

Auto-detected from file extensions. Same spec format for all.

| Language | Exports Detected | Test Exclusions |
|----------|-----------------|-----------------|
| **TypeScript/JS** | `export function/class/type/const/enum`, re-exports, `export *` wildcard resolution, CommonJS `export =` (AST supported) | `.test.ts`, `.spec.ts`, `.d.ts` |
| **Rust** | `pub` and `pub(crate)` fn/struct/enum/trait/type/const/static/mod; narrower restricted visibility excluded (AST supported) | `#[cfg(test)]` modules |
| **Go** | Uppercase `func/type/var/const`, methods | `_test.go` |
| **Python** | `__all__`, or top-level `def/class` (no `_` prefix) (AST supported) | `test_*.py`, `*_test.py` |
| **Swift** | `public/open` func/class/struct/enum/protocol/actor | `*Tests.swift` |
| **Kotlin** | Top-level declarations (excludes private/internal) | `*Test.kt`, `*Spec.kt` |
| **Java** | `public` class/interface/enum/record/methods | `*Test.java`, `*Tests.java` |
| **C#** | `public` class/struct/interface/enum/record/delegate | `*Test.cs`, `*Tests.cs` |
| **Dart** | Top-level (no `_` prefix), class/mixin/enum/typedef | `*_test.dart` |
| **PHP** | `class/interface/trait/enum`, `public` function/const, skips `private/protected` and `__` magic methods | `*Test.php` |
| **Ruby** | `class`/`module`, `public` methods with visibility tracking, `attr_accessor`/`attr_reader`/`attr_writer`, constants | `*_test.rb`, `*_spec.rb` |
| **YAML** | Top-level mapping keys (anchors, aliases supported) | — |
| **C** | Non-static function declarations/definitions, struct/union/enum names (AST supported) | `_test.c`, `test_` |
| **C++** | Public methods, classes, structs, unions, enums, namespaces (AST supported) | `_test.cpp`, `test_` |
| **Scala** | Classes, objects, traits, defs, vals, vars (AST supported) | `Spec.scala`, `Suite.scala`, `Test.scala` |
| **Crystal** | Classes, modules, structs, enums, defs, aliases | `_spec.cr` |
| **Nim** | Explicitly exported symbols marked with `*` | `t`, `test` |
| **Erlang** | Functions declared in `-export([...])` attributes (AST supported) | `_tests.erl` |
| **Elixir** | Public modules, functions, macros (AST supported) | `_test.exs` |
| **Perl** | All subroutine declarations (AST supported) | `_test.pl`, `.t` |
| **Lisp** | Dialects (Common Lisp, Scheme, Emacs Lisp) dispatched by extension: definitions, macros, structures (AST supported) | `test.lisp`, `test.scm` |
| **Haskell** | Exports in module header list, or all top-level symbols | `Spec.hs`, `Test.hs`, `Tests.hs` |
| **Lua** | Module tables (`local M = {}`), return tables, or top-level functions | `_spec.lua`, `_test.lua`, `spec.lua` |
| **R** | Roxygen2 `#' @export` tags, or top-level non-`.`-prefixed functions | `test-`, `test_` |
| **OCaml** | Top-level let bindings, types, modules, exceptions | `_test.ml`, `test_` |
| **Groovy** | Classes, interfaces, traits, enums, public methods | `Test.groovy`, `Tests.groovy`, `Spec.groovy` |
| **F#** | Let and type bindings | `Tests.fs`, `Test.fs`, `Spec.fs` |
| **Clojure** | Public defn/def/defrecord/deftype forms | `_test.clj`, `_test.cljs`, `_test.cljc` |
| **D** | Module-scope public classes, structs, functions | `_test.d`, `test_` |
| **Objective-C** | Class interfaces, protocols, and implementation methods | `Tests.m`, `Test.m`, `Spec.m` |
| **Bash** | `export -f` functions, or non-`_` functions | `_test.sh`, `test_` |
| **PowerShell** | `Export-ModuleMember` functions, or all functions | `Tests.ps1`, `.Tests.ps1` |
| **Vala** | Public classes, structs, interfaces, methods | `Test.vala`, `Tests.vala` |

---

## Install

### GitHub Action (recommended)

```yaml
- uses: CorvidLabs/spec-sync@v5
  with:
    strict: 'true'
    require-coverage: '100'
```

### Crates.io

```bash
cargo install specsync
```

### Pre-built binaries

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

### From source

```bash
cargo install --git https://github.com/CorvidLabs/spec-sync
```

---

## Get Started in 5 Minutes

Zero to a green spec check, on a fresh project. Copy-paste the whole block.

```bash
# 1. Install (one of the install methods above — we'll assume `cargo install specsync`)
cargo install specsync

# 2. Make a new project + a single Rust file
mkdir hello-spec && cd hello-spec
cargo init --lib --quiet
cat > src/lib.rs <<'EOF'
//! A trivial greeter.

/// Returns a greeting for `name`.
pub fn greet(name: &str) -> String {
    format!("hello, {name}!")
}
EOF

# 3. Initialize SpecSync (creates .specsync/config.toml + a registry)
specsync init

# 4. Scaffold a spec for the module. SpecSync auto-detects `src/lib.rs`
#    and generates a stub spec for the `greet` function it found.
specsync new greeter

# 5. Run the validator. The new spec has all required sections + matches code.
specsync check
```

Expected output:

```text
✓ greeter (v1, draft) — 1 source file, 7/7 sections
1 spec checked, 0 errors, 0 warnings
```

That's it. The spec is the contract; if you remove `pub fn greet` from the source, `specsync check` will fail. If you add a new public function and don't list it in the spec, you get a warning. Drift gets caught at CI time, not in code review.

**Next steps:**
- Add a `CorvidLabs/spec-sync@v5` GitHub Action to CI — it validates canonical specs and active change gates.
- Use `specsync wizard` for an interactive spec-creation experience instead of `new`.
- Use `specsync generate` for deterministic local scaffolds, then let your coding agent refine them.
- See [`examples/quickstart/`](./examples/quickstart) for the same flow as a committed reference.

For the full list of commands, see [CLI Reference](#cli-reference) below.

---

## CLI Reference

All commands at a glance. Run `specsync <command> --help` for details.

```bash
specsync migrate                           # Upgrade from 3.x to 4.0.0 (.specsync/ layout)
specsync migrate --dry-run                 # Preview migration without changes
specsync init                              # Create the .specsync/ v5 project layout
specsync change new "Add passkeys"         # Start the verified SDD interview
specsync change status                     # Show active delivery state and next actions
specsync change verify CHG-0001-add-passkeys # Run tests + requirement evidence gate
specsync change adopt --dry-run            # Preview 4.x/OpenSpec/Spec Kit adoption
specsync check                             # Validate specs against code
specsync check --fix                       # Auto-add undocumented exports as stubs
specsync diff                              # Show exports added/removed since HEAD
specsync diff --base HEAD~5                # Compare against a specific ref
specsync coverage                          # Show file/module coverage
specsync report                            # Per-module coverage with stale detection
specsync generate                          # Scaffold specs for unspecced modules
specsync agents install                     # Install native coding-agent workflows
specsync scaffold auth                     # Scaffold spec + auto-detect source files
specsync add-spec auth                     # Add a single spec + companion files
specsync deps                              # Validate cross-module dependency graph
specsync changelog v3.3.0..v3.4.0         # Generate changelog between git refs
specsync comment --pr 42                   # Post spec check summary as PR comment
specsync import github 123                 # Import spec from GitHub issue
specsync wizard                            # Interactive guided spec creation
specsync init-registry                     # Generate specsync-registry.toml
specsync resolve                           # Verify spec cross-references
specsync resolve --remote                  # Verify cross-project refs via GitHub
specsync score                             # Quality-score your spec files (0–100)
specsync compact --keep 10                 # Compact old changelog entries in specs
specsync archive-tasks                     # Archive completed tasks from tasks.md
specsync view --role dev                   # View specs filtered by role
specsync merge                             # Auto-resolve merge conflicts in specs
specsync new auth                          # Quick-create a minimal spec (auto-detects sources)
specsync stale                             # Find specs that haven't kept up with code changes
specsync rules                             # Show configured validation rules
specsync lifecycle status                  # Show lifecycle status of all specs
specsync lifecycle promote auth            # Advance auth spec to next status
specsync lifecycle history auth            # View transition history for a spec
specsync lifecycle guard auth              # Dry-run: check if guards would pass
specsync lifecycle auto-promote            # Promote all specs that pass guards
specsync lifecycle enforce --all           # CI: validate lifecycle rules
specsync hooks install                    # Install agent instructions + git hooks
specsync hooks status                     # Check what's installed
specsync agents install                   # Install native skills + create-spec/create-change commands
specsync agents status                    # Check what's installed
specsync mcp                               # Start MCP server for AI agent integration
specsync watch                             # Re-validate on every file change
```

---

## Spec Format

Specs are markdown files (`*.spec.md`) with YAML frontmatter in your specs directory.

### Frontmatter

```yaml
---
module: auth                                # Module name (required)
version: 3                                  # Spec version (required)
status: stable                              # draft | review | stable | deprecated (required)
files:                                      # Source files covered (required, non-empty)
  - src/auth/service.ts
  - src/auth/middleware.ts
db_tables:                                  # Validated against schema dir (optional)
  - users
  - sessions
depends_on:                                 # Other spec paths, validated for existence (optional)
  - specs/database/database.spec.md
  - corvid-labs/algochat@messaging           # Cross-project ref (owner/repo@module)
---
```

### Required Sections

Every spec must include these `##` sections (configurable in `.specsync/config.toml`):

Purpose, Public API, Invariants, Behavioral Examples, Error Cases, Dependencies, Change Log

> **Note:** Requirements (user stories, acceptance criteria) belong in a companion `requirements.md` file, not inline in the spec. Specs are technical contracts; requirements are product intent. See [Companion Files](#companion-files) below.

### Public API Tables

SpecSync extracts the first backtick-quoted name per row and cross-references it against code exports:

```markdown
## Public API

| Function | Parameters | Returns | Description |
|----------|-----------|---------|-------------|
| `authenticate` | `(token: string)` | `User \| null` | Validates bearer token |
| `refreshSession` | `(sessionId: string)` | `Session` | Extends session TTL |
```

<details>
<summary>Full spec example</summary>

```markdown
---
module: auth
version: 3
status: stable
files:
  - src/auth/service.ts
  - src/auth/middleware.ts
db_tables:
  - users
  - sessions
depends_on:
  - specs/database/database.spec.md
---

# Auth

## Purpose

Handles authentication and session management. Validates bearer tokens,
manages session lifecycle, provides middleware for route protection.

## Public API

### Exported Functions

| Function | Parameters | Returns | Description |
|----------|-----------|---------|-------------|
| `authenticate` | `(token: string)` | `User \| null` | Validates a token |
| `refreshSession` | `(sessionId: string)` | `Session` | Extends session TTL |

### Exported Types

| Type | Description |
|------|-------------|
| `User` | Authenticated user object |
| `Session` | Active session record |

## Invariants

1. Sessions expire after 24 hours
2. Failed auth attempts rate-limited to 5/minute
3. Tokens validated cryptographically, never by string comparison

## Behavioral Examples

### Scenario: Valid token

- **Given** a valid JWT token
- **When** `authenticate()` is called
- **Then** returns the corresponding User object

### Scenario: Expired token

- **Given** an expired JWT token
- **When** `authenticate()` is called
- **Then** returns null and logs a warning

## Error Cases

| Condition | Behavior |
|-----------|----------|
| Expired token | Returns null, logs warning |
| Malformed token | Returns null |
| DB unavailable | Throws `ServiceUnavailableError` |

## Dependencies

| Module | Usage |
|--------|-------|
| database | `query()` for user lookups |
| crypto | `verifyJwt()` for token validation |

## Change Log

| Date | Change |
|------|--------|
| 2026-03-18 | Initial spec |
```

</details>

---

## CLI Reference

```
specsync [command] [flags]
```

### Commands

| Command | Description |
|---------|-------------|
| `check` | Validate all specs against source code **(default)**. `--fix` auto-adds missing exports as stubs |
| `diff` | Show exports added/removed since a git ref (default: `HEAD`) |
| `coverage` | File and module coverage report |
| `report` | Per-module coverage report with stale and incomplete detection |
| `generate` | Deterministically scaffold specs for modules missing one |
| `scaffold <name>` | Scaffold spec + auto-detect source files + register in registry |
| `add-spec <name>` | Scaffold a single spec + companion files (`requirements.md`, `tasks.md`, `context.md`, `testing.md`, and `design.md` if enabled) |
| `deps` | Validate cross-module dependency graph (cycles, missing deps, undeclared imports) |
| `changelog <range>` | Generate changelog of spec changes between two git refs |
| `comment` | Post spec-sync check summary as a PR comment (same validation pipeline as `check`). `--pr N` to post, omit to print |
| `import <source> <id>` | Import specs from GitHub Issues, Jira, or Confluence |
| `wizard` | Interactive guided spec creation |
| `resolve` | Verify `depends_on` references exist. `--remote` fetches registries from GitHub |
| `init-registry` | Generate `specsync-registry.toml` from existing specs |
| `score` | Quality-score spec files (0–100) with improvement suggestions |
| `compact` | Compact old changelog entries in spec files. `--keep N` (default: 10) |
| `archive-tasks` | Archive completed tasks from companion `tasks.md` files |
| `view` | View specs filtered by role (`--role dev\|qa\|product\|agent`) |
| `merge` | Auto-resolve git merge conflicts in spec files |
| `new <name>` | Quick-create a minimal spec with auto-detected source files. `--full` includes companion files |
| `stale` | Identify specs that haven't been updated since their source files changed |
| `rules` | Display configured validation rules and built-in rule status |
| `migrate` | Upgrade from 3.x to 4.0.0 layout (`.specsync/` directory, TOML config). `--dry-run` to preview, `--no-backup` to skip |
| `lifecycle promote <spec>` | Advance spec to next status (draft→review→active→stable) |
| `lifecycle demote <spec>` | Step back one status level |
| `lifecycle set <spec> <status>` | Set spec to any status (with transition validation) |
| `lifecycle status [spec]` | Show lifecycle status of one or all specs |
| `lifecycle history <spec>` | Show transition history (audit log) for a spec |
| `lifecycle guard <spec> [target]` | Dry-run guard evaluation — check if transition would pass |
| `lifecycle auto-promote` | Promote all specs that pass their transition guards. `--dry-run` to preview |
| `lifecycle enforce` | CI enforcement — validate lifecycle rules, exit non-zero on violations. `--all` for all checks |
| `issues` | Verify GitHub issue references in spec frontmatter. `--create` to create missing issues |
| `hooks` | Install/uninstall agent instructions and git hooks (`install`, `uninstall`, `status`) |
| `agents` | Install/uninstall native AI-tool skills and slash commands for Claude Code, Cursor, Codex, and Gemini CLI (`install`, `uninstall`, `status`) |
| `mcp` | Start MCP server for AI agent integration (Claude Code, Cursor, etc.) |
| `init` | Create the default `.specsync/` 5.0 layout, config, and SDD policy |
| `watch` | Live validation on file changes (500ms debounce) |

### Flags

| Flag | Description |
|------|-------------|
| `--strict` | Warnings become errors (recommended for CI) |
| `--require-coverage N` | Fail if file coverage < N% |
| `--root <path>` | Project root (default: cwd) |
| `--fix` | Auto-add undocumented exports as stub rows in spec Public API tables |
| `--force` | Skip hash cache and re-validate all specs |
| `--create-issues` | Create GitHub issues for specs with validation errors (on `check`) |
| `--dry-run` | Preview changes without writing files (on `compact`, `archive-tasks`, `merge`) |
| `--stale N` | Flag specs N+ commits behind their source files (on `check`) |
| `--exclude-status <s>` | Exclude specs with the given status. Repeatable |
| `--only-status <s>` | Only process specs with the given status. Repeatable |
| `--mermaid` | Output dependency graph as Mermaid diagram (on `deps`) |
| `--dot` | Output dependency graph as Graphviz DOT (on `deps`) |
| `--full` | Include companion files (on `new`) |
| `--json` | Structured JSON output |

### Exit Codes

| Code | Meaning |
|------|---------|
| `0` | All checks passed |
| `1` | Errors, strict warnings, or coverage below threshold |

---

## Cross-Project References

Specs can declare dependencies on modules in other repositories using `owner/repo@module` syntax in `depends_on`:

```yaml
depends_on:
  - specs/database/database.spec.md       # Local reference
  - corvid-labs/algochat@messaging         # Cross-project reference
```

### Registry

Each project publishes a `specsync-registry.toml` at its root to declare available spec modules:

```toml
[registry]
name = "myapp"

[specs]
auth = "specs/auth/auth.spec.md"
messaging = "specs/messaging/messaging.spec.md"
database = "specs/db/database.spec.md"
```

Generate one automatically from existing specs:

```bash
specsync init-registry                    # Uses project folder name
specsync init-registry --name myapp       # Custom registry name
```

### Resolving References

```bash
specsync resolve                          # Verify local refs exist
specsync resolve --remote                 # Also verify cross-project refs via GitHub
```

Remote resolution fetches `specsync-registry.toml` from each referenced repo and validates that the module exists. Requests are grouped by repo to minimize HTTP calls.

**Zero CI cost by default** — `specsync check` validates local refs only (no network). Use `--remote` explicitly when you want cross-project verification.

---

## Companion Files

When you run `specsync generate` or `specsync add-spec`, four companion files are created alongside each spec, plus one opt-in:

| File | Author | Validated? | Purpose |
|------|--------|-----------|---------|
| `{module}.spec.md` | Dev/Architect | Yes — against code | Technical contract |
| `tasks.md` | Anyone | No | Work coordination |
| `context.md` | Dev/Agent | No | Architecture notes |
| `requirements.md` | Product/Design | No | The ask, acceptance criteria |
| `testing.md` | QA/Dev | No | Test strategy, edge cases, manual checklists |
| `design.md` *(opt-in)* | Design/Frontend | No | Layout, component hierarchy, design tokens |

`design.md` is opt-in — enable it in `.specsync/config.toml`:

```toml
[companions]
design = true
```

All scaffolded by SpecSync, all human-filled. Only the spec gets bidirectional validation.

> **Convention:** Requirements (user stories, acceptance criteria) must live in `requirements.md`, not as inline `## Requirements` sections inside the spec. Specs define the *technical contract*; requirements capture *product intent*. Inline requirements in non-draft specs produce a warning prompting you to move them to the companion file.

**`requirements.md`** — Product requirements and acceptance criteria:

```markdown
---
spec: auth.spec.md
---

## User Stories
- As a maintainer, I want this module's contract captured clearly so changes can be reviewed against stable behavior.

## Acceptance Criteria
- Define acceptance criteria from the module's source behavior and user-facing responsibilities.

## Constraints
<!-- Non-functional requirements, performance targets, compliance needs -->

## Out of Scope
<!-- Explicitly excluded from this module's requirements -->
```

**`tasks.md`** — Multi-role checkpoint tracking:

```markdown
---
spec: auth.spec.md
---

## Tasks
- [ ] <!-- Implementation checklist -->

## Gaps
Record concrete coverage gaps or edge cases that still need tests.

## Review Sign-offs
- **Product**: pending
- **QA**: pending
- **Design**: n/a
- **Dev**: pending
```

**`context.md`** — Agent briefing document:

```markdown
---
spec: auth.spec.md
---

## Key Decisions
<!-- Architectural or design decisions -->

## Files to Read First
<!-- Most important files for understanding this module -->

## Current Status
<!-- What's done, in progress, blocked -->

## Notes
<!-- Free-form notes, links, context -->
```

**`testing.md`** — Test strategy and QA checklist:

```markdown
---
spec: auth.spec.md
---

## Automated Testing
<!-- Test file locations and what they cover -->

## Manual QA Checklist
- [ ] <!-- Manual verification steps -->

## Edge Cases
<!-- Boundary conditions, race conditions, unusual inputs -->
```

**`design.md`** *(opt-in)* — Design and layout documentation:

```markdown
---
spec: auth.spec.md
---

## Layout
<!-- Page/component layout, responsive breakpoints, positioning -->

## Components
<!-- Component tree, props, slots -->

## Tokens
<!-- Design token overrides from global design system -->

## Assets
<!-- Icons, images, illustrations needed -->
```

These files are designed for team coordination and AI agent context — they give any contributor (human or AI) the full picture of where a module stands.

---

## YAML Source Files

SpecSync extracts symbols from YAML files (`.yml`, `.yaml`) including GitHub Actions workflows, Docker Compose files, and Kubernetes manifests.

**What gets extracted:**
- **Top-level keys** — any key at column 0 (e.g., `name`, `on`, `jobs`)
- **Nested children of well-known parents** — `jobs.test`, `services.web`, `volumes.pgdata`, etc. Parents: `jobs`, `services`, `volumes`, `networks`, `secrets`, `stages`, `steps`, `targets`, `outputs`, `inputs`, `permissions`, `deployments`
- **YAML anchors** — `&anchor-name` definitions

**Indentation:** Any consistent indentation (2-space, 4-space, or tabs) is supported for nested key extraction.

### Document Separator Edge Cases

YAML supports multi-document files separated by `---` (common in Kubernetes manifests). SpecSync handles these correctly:

```yaml
# Multi-document YAML (e.g., k8s manifests)
apiVersion: apps/v1
kind: Deployment
metadata:
  name: my-app
---
apiVersion: v1
kind: Service
metadata:
  name: my-svc
```

- Top-level keys from **all documents** are extracted (`apiVersion`, `kind`, `metadata` from both)
- The `---` separator is not confused with spec frontmatter delimiters — frontmatter parsing only applies to `.spec.md` files
- Duplicate keys across documents are deduplicated in the symbol list

> **Note:** SpecSync's YAML extractor uses regex-based parsing, not a full YAML parser. This means it works without any YAML library dependency but does not interpret YAML semantics like merge keys (`<<: *alias`) beyond anchor extraction.

---

## VS Code Extension

Install from the [VS Code Marketplace](https://marketplace.visualstudio.com/items?itemName=corvidlabs.specsync) or search "SpecSync" in the Extensions panel.

```bash
code --install-extension corvidlabs.specsync
```

### Features

| Feature | Description |
|---------|-------------|
| **Inline diagnostics** | Errors and warnings mapped to spec files on save |
| **CodeLens scores** | Quality scores (0–100) displayed inline above spec files |
| **Coverage report** | Rich webview with file and LOC coverage |
| **Scoring report** | Per-spec quality breakdown with improvement suggestions |
| **Status bar** | Persistent pass/fail/error indicator |
| **Validate-on-save** | Automatic validation with 500ms debounce |

### Commands (Command Palette)

- `SpecSync: Validate Specs` — run `specsync check`
- `SpecSync: Show Coverage` — open coverage report
- `SpecSync: Score Spec Quality` — open scoring report
- `SpecSync: Generate Missing Specs` — scaffold specs for unspecced modules
- `SpecSync: Initialize Config` — create the `.specsync/` 5.0 project layout

### Settings

| Setting | Default | Description |
|---------|---------|-------------|
| `specsync.binaryPath` | `specsync` | Path to the specsync binary |
| `specsync.validateOnSave` | `true` | Run validation on file save |
| `specsync.showInlineScores` | `true` | Show CodeLens quality scores |

The extension activates automatically in workspaces containing `.specsync/config.toml`, legacy `specsync.json`/`.specsync.toml`, or a `specs/` directory. Requires the `specsync` CLI binary to be installed and on your PATH (or configured via `specsync.binaryPath`).

---

## GitHub Action

Available on the [GitHub Marketplace](https://github.com/marketplace/actions/spec-sync). Auto-detects OS/arch, downloads the binary, runs validation.

### Inputs

| Input | Default | Description |
|-------|---------|-------------|
| `version` | `latest` | Release version to download |
| `strict` | `false` | Treat warnings as errors |
| `require-coverage` | `0` | Minimum file coverage % |
| `root` | `.` | Project root directory |
| `args` | `''` | Extra whitespace-separated CLI arguments; shell quoting is not supported |
| `comment` | `false` | Post spec drift results as a PR comment (requires `pull_request` event) |
| `token` | `${{ github.token }}` | GitHub token for posting PR comments (needs write permissions) |

### Workflow examples

**Basic CI check:**

```yaml
name: Spec Check
on: [push, pull_request]

jobs:
  specsync:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v5
        with:
          fetch-depth: 0
      - uses: CorvidLabs/spec-sync@v5
        with:
          strict: 'true'
          require-coverage: '100'
```

**With PR comments:**

```yaml
name: Spec Check
on:
  pull_request:
    types: [opened, synchronize]

jobs:
  specsync:
    runs-on: ubuntu-latest
    permissions:
      pull-requests: write
    steps:
      - uses: actions/checkout@v5
        with:
          fetch-depth: 0
      - uses: CorvidLabs/spec-sync@v5
        with:
          strict: 'true'
          comment: 'true'
```

When `comment: 'true'` is set, SpecSync posts (or updates) a PR comment showing spec drift — added/removed exports since the base branch. The comment is automatically updated on subsequent pushes.

---

## Configuration

Run `specsync init` to create `.specsync/config.toml`. Legacy `specsync.json` and `.specsync.toml` files remain readable for migration:

```toml
specs_dir = "specs"
source_dirs = ["src"]
schema_dir = "db/migrations"
required_sections = ["Purpose", "Public API", "Invariants", "Behavioral Examples", "Error Cases", "Dependencies", "Change Log"]
exclude_dirs = ["__tests__"]
exclude_patterns = ["**/__tests__/**", "**/*.test.ts", "**/*.spec.ts"]
source_extensions = []
```

| Option | Type | Default | Description |
|--------|------|---------|-------------|
| `specs_dir` | `string` | `"specs"` | Directory containing `*.spec.md` files |
| `source_dirs` | `string[]` | `["src"]` | Source directories for coverage analysis |
| `schema_dir` | `string?` | — | SQL schema dir for `db_tables` validation |
| `schema_pattern` | `string?` | `CREATE TABLE` regex | Custom regex for table name extraction |
| `required_sections` | `string[]` | 7 defaults | Markdown sections every spec must include |
| `exclude_dirs` | `string[]` | `["__tests__"]` | Directories excluded from coverage |
| `exclude_patterns` | `string[]` | Common test globs | File patterns excluded from coverage |
| `source_extensions` | `string[]` | All supported | Restrict to specific extensions (e.g., `["ts", "rs"]`) |
| `parse_mode` | `string` | `"regex"` | Parser backend: `"regex"` (default) or `"ast"` (tree-sitter, opt-in for TypeScript, Python, Rust, C, C++, Scala, Erlang, Elixir, Perl, Lisp) |

### Legacy root-level TOML

```toml
# .specsync.toml (legacy; migrate to .specsync/config.toml)
specs_dir = "specs"
source_dirs = ["src", "lib"]
required_sections = ["Purpose", "Public API", "Invariants", "Behavioral Examples", "Error Cases", "Dependencies", "Change Log"]
```

Config resolution order: `.specsync/config.toml` → `.specsync/config.json` → `.specsync.toml` → `specsync.json` → defaults with auto-detected source dirs.

### Agent-native enrichment

SpecSync 5.0 never accepts provider credentials, sends source to a model, or executes an AI command. Run `specsync generate` for a deterministic local scaffold, then use `specsync agents install` or the MCP server so your existing coding agent can refine the markdown inside that agent's own trust boundary. Legacy AI configuration keys are ignored with migration guidance.

### Lifecycle Guards

Configure transition guards in `.specsync/config.toml` to enforce quality gates before specs can be promoted:

```toml
[lifecycle]
track_history = true

[lifecycle.guards."review→active"]
min_score = 70
require_sections = ["Public API", "Invariants"]

[lifecycle.guards."active→stable"]
min_score = 85
no_stale = true
require_sections = ["Public API", "Behavioral Examples", "Error Cases"]

[lifecycle.guards."*→stable"]
min_score = 85
message = "Stable specs require high quality scores"
```

| Guard Option | Type | Description |
|-------------|------|-------------|
| `minScore` | `number?` | Minimum spec quality score (0-100) required |
| `requireSections` | `string[]` | Sections that must exist with non-empty content |
| `noStale` | `bool?` | Spec must not be stale (source files ahead of spec) |
| `staleThreshold` | `number?` | Max commits behind when `noStale` is true (default: 5) |
| `message` | `string?` | Custom message shown when guard blocks transition |

Guard keys use `"from→to"` format (e.g., `"review→active"`) or `"*→to"` for wildcard. ASCII arrows (`->`) also work.

When `trackHistory` is enabled (default: `true`), every status transition is recorded in the spec's frontmatter:

```yaml
lifecycle_log:
  - "2026-04-11: draft → review"
  - "2026-04-12: review → active"
```

Use `specsync lifecycle guard <spec>` to dry-run guard evaluation without making changes.

### Auto-Promote & CI Enforcement

**Auto-promote** scans all specs and promotes any whose next transition passes all configured guards:

```bash
specsync lifecycle auto-promote            # promote eligible specs
specsync lifecycle auto-promote --dry-run  # preview without modifying
```

**Enforce** validates lifecycle rules for CI pipelines (exits non-zero on violations):

```bash
specsync lifecycle enforce --all           # run all checks
specsync lifecycle enforce --require-status # every spec needs a status field
specsync lifecycle enforce --max-age       # flag stale statuses
specsync lifecycle enforce --allowed       # check allowed statuses
```

Configure enforcement rules in `.specsync/config.toml`:

```toml
[lifecycle]
allowed_statuses = ["draft", "review", "active", "stable"]

[lifecycle.max_age]
draft = 30
review = 14
```

| Config Key | Type | Description |
|-----------|------|-------------|
| `max_age` | `table` | Maximum days a spec may stay in each status (e.g., `draft = 30`) |
| `allowed_statuses` | `string[]` | Restrict specs to these statuses only |

**GitHub Action** — add `lifecycle-enforce: 'true'` to the spec-sync action to enforce lifecycle rules in CI:

```yaml
- uses: CorvidLabs/spec-sync@v5
  with:
    lifecycle-enforce: 'true'
```

---

## Spec Generation

`specsync generate` scans your source directories, finds modules without spec files, and scaffolds `*.spec.md` files for each one.

```bash
specsync generate                         # Scaffold template specs for all unspecced modules
specsync coverage                         # See what's still missing
```

### Template mode (default)

Uses your custom template (`specs/_template.spec.md`) or the built-in default. Generates frontmatter plus guided starter sections for review.

### Deterministic core, agent-native workflow

Generation reads local source metadata and writes reviewable templates; it performs no network inference and needs no API key. Install the native integration for Claude Code, Cursor, Codex, or Gemini, or expose the deterministic tools over MCP. The invoking coding agent can then read source and enrich the scaffold under its own permissions and audit boundary.

### Designed for AI agents

For Claude Code, Cursor, Codex, and Gemini CLI, `specsync agents install` writes a project `SKILL.md` in each tool's documented discovery location. Claude, Cursor, and Gemini also receive create-spec and create-change commands; Codex gets the project skill only because its command mechanism is deprecated and global-only. Local installation and content are tested; live model discovery remains a separate environment/authentication check. This is separate from `specsync hooks install`, which remains the prose-instruction-file mechanism (`CLAUDE.md`, `.cursorrules`, `AGENTS.md`, Copilot instructions) — run both, or whichever applies to your setup.

The deterministic generator and validator are the entry points for agent workflows:

```bash
specsync generate                                  # Write deterministic local scaffolds
specsync agents install                            # Install native agent instructions
specsync check --fix                               # auto-add any missing exports as stubs
specsync check --json                              # validate, get structured feedback
# LLM fixes errors from JSON output                # iterate until clean
specsync check --strict --require-coverage 100     # enforce full coverage in CI
```

Every output format is designed for machine consumption:
- **`--json`** on any command → structured JSON, no ANSI codes
- **Exit code 0/1** → pass/fail, no parsing needed
- **Spec files are plain markdown** → any LLM can read and write them
- **Public API tables** use backtick-quoted names → unambiguous to extract

### JSON output shapes

```json
// specsync check --json
{ "passed": false, "errors": ["..."], "warnings": ["..."], "specs_checked": 12 }

// specsync coverage --json
{ "file_coverage": 85.33, "files_covered": 23, "files_total": 27, "loc_coverage": 79.12, "loc_covered": 4200, "loc_total": 5308, "modules": [...] }

// specsync diff --base HEAD~3 --json
{ "added": ["newFunction", "NewType"], "removed": ["oldHelper"], "spec": "specs/auth/auth.spec.md" }
```

---

## Auto-Fix & Diff

### `--fix`: Keep specs in sync automatically

```bash
specsync check --fix              # Add undocumented exports as stub rows
specsync check --fix --json       # Same, with structured JSON output
```

When `--fix` is used, any export found in code but missing from the spec gets appended as a review row with the symbol name and a description prompt. If no `## Public API` section exists, one is created. Already-documented exports are never duplicated.

This turns spec maintenance from manual table editing into a review-and-refine workflow — run `--fix`, then replace the generated prompts with final descriptions.

### `diff`: Track API changes across commits

```bash
specsync diff                     # Changes since HEAD (staged + unstaged)
specsync diff --base HEAD~5       # Changes since 5 commits ago
specsync diff --base v2.1.0       # Changes since a tag
```

Shows exports added and removed per spec file since the given git ref. Useful for code review, release notes, and CI drift detection.

---

## Architecture

```
src/
├── main.rs            CLI entry + output formatting
├── agents.rs          Native skill/slash-command installation (Claude Code, Cursor, Codex, Gemini CLI)
├── archive.rs         Task archival from companion tasks.md files
├── changelog.rs       Changelog generation between git refs
├── comment.rs         PR comment generation with spec links
├── compact.rs         Changelog entry compaction in spec files
├── config.rs          v4 and legacy config loading
├── deps.rs            Cross-module dependency graph validation
├── generator.rs       Spec + companion file scaffolding
├── github.rs          GitHub API integration (issues, PRs)
├── hash_cache.rs      Incremental validation via content hashing
├── hooks.rs           Agent instruction + git hook management
├── importer.rs        External importers (GitHub Issues, Jira, Confluence)
├── lifecycle.rs       Spec status transitions (promote, demote, set, status, history, guard, auto-promote, enforce)
├── manifest.rs        Package manifest parsing (Cargo.toml, package.json, etc.)
├── mcp.rs             MCP server for AI agent integration (JSON-RPC stdio)
├── merge.rs           Auto-resolve merge conflicts in spec files
├── new.rs             Quick-create minimal spec with source auto-detection
├── parser.rs          Frontmatter + spec body parsing
├── rules.rs           Display configured validation rules
├── registry.rs        Registry loading, generation, and remote fetching
├── schema.rs          SQL schema parsing for column validation
├── scoring.rs         Spec quality scoring (0–100, weighted rubric)
├── stale.rs           Staleness detection (spec vs source modification)
├── types.rs           Data types + config schema
├── validator.rs       Validation + coverage + cross-project ref detection
├── view.rs            Role-filtered spec viewing (dev, qa, product, agent)
├── watch.rs           File watcher (notify, 500ms debounce)
└── exports/
    ├── mod.rs          Language dispatch
    ├── typescript.rs   TS/JS exports
    ├── rust_lang.rs    Rust pub items
    ├── go.rs           Go uppercase identifiers
    ├── python.rs       Python __all__ / top-level
    ├── swift.rs        Swift public/open items
    ├── kotlin.rs       Kotlin top-level
    ├── java.rs         Java public items
    ├── csharp.rs       C# public items
    ├── dart.rs         Dart public items
    ├── php.rs          PHP public items
    ├── ruby.rs         Ruby public items
    └── yaml.rs         YAML top-level keys
```

**Design:** Single binary, no runtime deps. Frontmatter parsed with regex (no YAML library). Language backends use regex, not ASTs — works without compilers installed. Release builds use LTO + strip + opt-level 3.

---

## Contributing

1. Fork, branch (`git checkout -b feat/my-feature`), implement
2. `cargo test` + `cargo clippy`
3. Open a PR

### Adding a language

1. Create `src/exports/yourlang.rs` — return `Vec<String>` of exported names
2. Add variant to `Language` enum in `types.rs`
3. Wire extension detection + dispatch in `src/exports/mod.rs`
4. Add tests for common patterns

---

## License

[MIT](LICENSE) &copy; [CorvidLabs](https://github.com/CorvidLabs)
