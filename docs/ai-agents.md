---
title: For AI Agents
layout: default
nav_order: 6
---

# For AI Agents
{: .no_toc }

SpecSync is built for LLM-powered coding tools — structured output, machine-readable specs, and automated scaffolding.
{: .fs-6 .fw-300 }

<details open markdown="block">
  <summary>Table of contents</summary>
  {: .text-delta }
1. TOC
{:toc}
</details>

---

## Generate Specs Automatically

`specsync generate` is the starting point for AI-driven spec workflows. It finds every module without a spec file and scaffolds one — frontmatter populated, source files listed, required sections stubbed with TODOs.

```bash
specsync generate
#   ✓ Generated specs/auth/auth.spec.md (3 files)
#   ✓ Generated specs/payments/payments.spec.md (2 files)
```

### What gets generated

For each unspecced module, you get a ready-to-fill spec:

```yaml
---
module: auth
version: 1
status: draft
files:
  - src/auth/service.ts
  - src/auth/middleware.ts
db_tables: []
depends_on: []
---
```

Plus all required sections (Purpose, Public API, Invariants, Behavioral Examples, Error Cases, Dependencies, Change Log) with TODO placeholders.

### Custom templates

Place `_template.spec.md` in your specs directory to control the generated structure. The generator replaces `module`, `version`, `status`, `files`, and the `# Title` heading — your template controls everything else.

---

## End-to-End AI Workflow

```bash
# 1. Bootstrap: scaffold specs for all unspecced modules
specsync generate

# 2. Fill: LLM reads source code, fills in each spec's content
#    (Purpose, Public API tables, Invariants, etc.)

# 3. Validate: check specs against code, get structured errors
specsync check --json

# 4. Fix: LLM reads JSON errors, corrects specs or flags code issues

# 5. Enforce: CI gate with full coverage
specsync check --strict --require-coverage 100
```

Each step produces machine-readable output. No human in the loop required (though humans can review at any step).

---

## Why SpecSync Works for LLMs

| Feature | Why it matters |
|---------|---------------|
| Plain markdown specs | Any LLM can read and write them — no custom format to learn |
| `--json` flag on every command | Structured output, no ANSI codes to strip |
| Exit code 0/1 | Pass/fail without parsing |
| Backtick-quoted names in API tables | Unambiguous extraction — first backtick-quoted string per row |
| `specsync generate` | Bootstrap from zero — LLM fills in content, not boilerplate |
| Deterministic validation | Same input → same output, no flaky checks |

---

## JSON Output Shapes

### `specsync check --json`

```json
{
  "passed": false,
  "errors": ["auth.spec.md: phantom export `oldFunction` not found in source"],
  "warnings": ["auth.spec.md: undocumented export `newHelper`"],
  "specs_checked": 12
}
```

- **Errors**: spec references something missing from code — must fix
- **Warnings**: code exports something the spec doesn't mention — informational
- **`--strict`**: promotes warnings to errors

### `specsync coverage --json`

```json
{
  "file_coverage": 85.33,
  "files_covered": 23,
  "files_total": 27,
  "modules": [{ "name": "helpers", "has_spec": false }]
}
```

Use `modules` with `has_spec: false` to identify what `generate` would scaffold.

---

## Writing Specs Programmatically

1. Frontmatter requires `module`, `version`, `status`, `files`
2. Status values: `draft`, `review`, `stable`, `deprecated`
3. `files` must be non-empty, paths relative to project root
4. Public API tables: first backtick-quoted string per row is the export name
5. Default required sections: Purpose, Public API, Invariants, Behavioral Examples, Error Cases, Dependencies, Change Log

### Minimal valid spec

```markdown
---
module: mymodule
version: 1
status: draft
files:
  - src/mymodule.ts
---

# MyModule

## Purpose
TODO

## Public API

| Export | Description |
|--------|-------------|
| `myFunction` | Does something |

## Invariants
TODO

## Behavioral Examples
TODO

## Error Cases
TODO

## Dependencies
None

## Change Log

| Date | Change |
|------|--------|
| 2026-03-19 | Initial spec |
```

---

## Integration Patterns

| Pattern | Command | How |
|---------|---------|-----|
| **Pre-commit hook** | `specsync check --strict` | Block commits with spec errors |
| **PR review bot** | `specsync check --json` | Parse output, post as PR comment |
| **Bootstrap coverage** | `specsync generate` | Scaffold after adding new modules |
| **AI code review** | `specsync check --json` | Feed errors to LLM for spec updates |
| **Coverage gate** | `specsync check --strict --require-coverage 100` | CI enforces full coverage |
