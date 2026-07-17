---
title: "Quick Start Guide"
section: "Getting started"
order: 2
---

Get SpecSync running on your project in under 5 minutes.

---

## Install

Choose your preferred method:

```bash
# Via cargo (recommended)
cargo install specsync

# Via Homebrew
brew install CorvidLabs/tap/spec-sync

# Via GitHub releases (no Rust toolchain needed)
# Download the binary for your platform from:
# https://github.com/CorvidLabs/spec-sync/releases

# Via GitHub Action (CI only)
# See integrations/github-action
```

Verify the installation:

```bash
specsync --version
```

---

## 1. Initialize Your Project

Navigate to your project root and run:

```bash
specsync init
```

This creates `.specsync/config.toml`, enables the verified lifecycle in `.specsync/sdd.json`, detects test commands, and offers to install native agent skills and create the first change.

```toml
specs_dir = "specs"
source_dirs = ["src"]
required_sections = [
    "Purpose",
    "Public API",
    "Invariants",
    "Behavioral Examples",
    "Error Cases",
    "Dependencies",
    "Change Log",
]
```

**Key settings:**
- `specs_dir` — where spec files live (default: `specs/`)
- `source_dirs` — where your source code lives (auto-detected from package manifests)
- `required_sections` — what every spec must contain

See [Configuration](configuration.md) for all options.

---

## 2. Create a Verified Change

```bash
specsync change new "Document and verify the existing authentication module" \
  --spec auth --path src/auth
```

Answer the returned questions, complete its adaptively selected artifacts, and approve the definition before implementation. Agents installed during `init` conduct this interview conversationally.

> **Two gates fire before `change approve` succeeds:**
>
> 1. **Artifact completeness** — every generated artifact in the change workspace (`context.md`, `requirements.md`, `tasks.md`, `testing.md`, and any others the interview selected) must have its scaffold `TODO` marker replaced with real content. Approval fails with a path-and-line diagnostic naming the unfinished artifact.
> 2. **Semantic deltas** — each spec module the change touches needs a delta file at `.specsync/changes/<change-id>/deltas/<module>.md` describing its contract changes (added/modified requirements and spec-section content). The set of delta modules must exactly match the change's affected specs, or approval fails with `semantic delta modules must exactly match affected specs`.

Continue through the same lifecycle after the definition is complete:

```bash
specsync change approve CHG-...   # explicit human definition approval
specsync change start CHG-...     # begin implementation
# implement the approved contract and keep module specs synchronized
specsync change verify CHG-...    # run configured tests and record evidence
specsync change accept CHG-...    # explicit human closing approval
# merge the delivery branch, then archive its immutable change record
specsync change archive CHG-...
```

See the [Workflow Guide](workflow.md) for requirements, semantic deltas, approval digests, verification evidence, and archival rules.

## 3. Generate Specs

Generate template specs for all source modules:

```bash
# Deterministic local scaffold
specsync generate
```

This creates a directory structure like:

```
specs/
├── auth/
│   ├── auth.spec.md        ← The spec (validated)
│   ├── requirements.md     ← User stories & acceptance criteria
│   ├── tasks.md            ← Work items & sign-offs
│   ├── context.md          ← Architecture notes & key files
│   ├── testing.md          ← Test strategy & QA checklist
│   └── design.md           ← (opt-in) Layout & design tokens
├── database/
│   ├── database.spec.md
│   ├── requirements.md
│   ├── tasks.md
│   ├── context.md
│   └── testing.md
└── ...
```

Each `.spec.md` file has YAML frontmatter and required sections:

```markdown
---
module: auth
version: 1.0.0
status: draft
files:
  - src/auth.ts
  - src/auth.utils.ts
---

## Purpose
Handles user authentication via JWT tokens.

## Public API
| Export | Type | Description |
|--------|------|-------------|
| `login(email, password)` | function | Authenticates a user |
| `logout(token)` | function | Invalidates a session |
| `AuthConfig` | interface | Configuration options |

## Invariants
- Tokens expire after 24 hours
- Failed login attempts are rate-limited

## Behavioral Examples
...
```

---

## 4. Validate

Run validation to check specs against your code:

```bash
specsync check
```

You'll see output like:

```
✓ specs/auth/auth.spec.md
✗ specs/database/database.spec.md
  ERROR: Spec references `createUser` but export not found in src/database.ts
  WARNING: `deleteUser` exported from code but not documented in spec

1 passed, 1 failed (2 errors, 1 warning)
File coverage: 85.7% (6/7 files)
```

**Errors** mean the spec claims something exists that doesn't. **Warnings** mean the code has something the spec doesn't mention yet.

### Strict Mode

In CI, use strict mode to fail on warnings too:

```bash
specsync check --strict
```

### Coverage Threshold

Require a minimum percentage of source files to have specs:

```bash
specsync check --require-coverage 80
```

---

## 5. Iterate

Fix the issues SpecSync found:

1. **Export renamed?** Update the spec's Public API table
2. **New export not in spec?** Add it to the table
3. **Deleted file?** Remove it from the spec's `files` list or archive the spec

Then run `specsync check` again until everything passes.

---

## 6. Add to CI

### GitHub Action

```yaml
# .github/workflows/specsync.yml
name: SpecSync
on: [push, pull_request]

jobs:
  check:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v5
        with:
          fetch-depth: 0
      - uses: CorvidLabs/spec-sync@v5.1.1
        with:
          version: '5.1.1'
          strict: true
          require-coverage: 80
```

### Manual CI

```bash
# In any CI system
cargo install specsync
specsync check --strict --require-coverage 80
```

---

## What's Next?

Once you're up and running, explore these features:

| Feature | Command | Guide |
|---------|---------|-------|
| Quality scoring | `specsync score` | [CLI Reference](cli.md#score) |
| Watch mode | `specsync watch` | [CLI Reference](cli.md#watch) |
| Agent enrichment | `specsync agents install` | [AI Agents](integrations/ai-agents.md) |
| Schema validation | Add `schema_dir` to config | [Configuration](configuration.md) |
| Cross-project refs | `owner/repo@module` syntax | [Cross-Project Refs](cross-project-refs.md) |
| MCP server | `specsync mcp` | [AI Agents](integrations/ai-agents.md) |
| VS Code extension | Install from marketplace | [VS Code Extension](integrations/vscode-extension.md) |
| Agent instructions | `specsync hooks` | [CLI Reference](cli.md#hooks) |
| Merge conflicts | `specsync merge` | [CLI Reference](cli.md#merge) |

For the full lifecycle guide (create → validate → iterate → stabilize → maintain → compact → archive), see the [Workflow Guide](workflow.md).
