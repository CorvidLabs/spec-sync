# Quick Start Guide

Get SpecSync running on your project in under 5 minutes.

---

## Install

Choose your preferred method:

```bash
# Via cargo (recommended)
cargo install specsync

# Via GitHub releases (no Rust toolchain needed)
# Download the binary for your platform from:
# https://github.com/CorvidLabs/spec-sync/releases

# Via GitHub Action (CI only)
# See github-action.md
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

This creates `.specsync/config.toml` with auto-detected source directories and adds `.specsync/hashes.json` to your `.gitignore` (the hash cache is a local-only optimization). The config looks like:

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

## 2. Generate Specs

Generate template specs for all source modules:

```bash
# Template-based (instant, no AI needed)
specsync generate

# AI-powered (richer content, requires AI provider)
specsync generate --ai
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

# Purpose
Handles user authentication via JWT tokens.

# Public API
| Export | Type | Description |
|--------|------|-------------|
| `login(email, password)` | function | Authenticates a user |
| `logout(token)` | function | Invalidates a session |
| `AuthConfig` | interface | Configuration options |

# Invariants
- Tokens expire after 24 hours
- Failed login attempts are rate-limited

# Behavioral Examples
...
```

---

## 3. Validate

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

## 4. Iterate

Fix the issues SpecSync found:

1. **Export renamed?** Update the spec's Public API table
2. **New export not in spec?** Add it to the table
3. **Deleted file?** Remove it from the spec's `files` list or archive the spec

Then run `specsync check` again until everything passes.

---

## 5. Add to CI

### GitHub Action

```yaml
# .github/workflows/specsync.yml
name: SpecSync
on: [push, pull_request]

jobs:
  check:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - uses: CorvidLabs/spec-sync@v4
        with:
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
| AI generation | `specsync generate --ai` | [AI Agents](ai-agents.md) |
| Schema validation | Add `schemaDir` to config | [Configuration](configuration.md) |
| Cross-project refs | `owner/repo@module` syntax | [Cross-Project Refs](cross-project-refs.md) |
| MCP server | `specsync mcp` | [AI Agents](ai-agents.md) |
| VS Code extension | Install from marketplace | [VS Code Extension](vscode-extension.md) |
| Agent instructions | `specsync hooks` | [CLI Reference](cli.md#hooks) |
| Merge conflicts | `specsync merge` | [CLI Reference](cli.md#merge) |

For the full lifecycle guide (create → validate → iterate → stabilize → maintain → compact → archive), see the [Workflow Guide](workflow.md).
