---
title: CLI Reference
layout: default
nav_order: 3
---

# CLI Reference
{: .no_toc }

<details open markdown="block">
  <summary>Table of contents</summary>
  {: .text-delta }
1. TOC
{:toc}
</details>

---

## Usage

```
specsync [command] [flags]
```

If no command is given, `check` runs by default.

---

## Commands

### `check`

Validate all specs against source code. This is the default command.

```bash
specsync check
specsync check --strict
specsync check --strict --require-coverage 100
specsync check --json
```

Runs three levels of validation:
1. **Structural** — required frontmatter fields, file existence, required markdown sections
2. **API surface** — cross-references spec symbols against actual code exports
3. **Dependencies** — validates `depends_on` paths and `db_tables` against schema

### `coverage`

Show a file and module coverage report — which modules have specs and which don't.

```bash
specsync coverage
specsync coverage --json
```

### `generate`

Scaffold spec files for modules that don't have one yet. Uses `specs/_template.spec.md` if it exists, otherwise a built-in template.

```bash
specsync generate
```

### `init`

Create a default `specsync.json` configuration file in the current directory.

```bash
specsync init
```

### `watch`

Live validation mode. Watches your specs and source directories for changes and re-runs validation automatically with a 500ms debounce.

```bash
specsync watch
```

Press `Ctrl+C` to exit.

---

## Flags

| Flag | Description |
|:-----|:------------|
| `--strict` | Treat warnings as errors. Recommended for CI. |
| `--require-coverage N` | Fail if file coverage is below N percent. |
| `--root <path>` | Set project root directory (default: current directory). |
| `--json` | Output structured JSON instead of colored terminal text. |

---

## Exit Codes

| Code | Meaning |
|:-----|:--------|
| `0` | All checks passed |
| `1` | Errors found, warnings found with `--strict`, or coverage below `--require-coverage` threshold |

---

## JSON Output

Use `--json` for machine-readable output. Useful for CI pipelines, editor integrations, and AI agents.

### Check output

```json
{
  "passed": false,
  "errors": [
    "auth.spec.md: Spec documents 'oldFunction' but no matching export found in source"
  ],
  "warnings": [
    "auth.spec.md: Export 'newHelper' not in spec (undocumented)"
  ],
  "specs_checked": 12
}
```

### Coverage output

```json
{
  "file_coverage": 85.33,
  "files_covered": 23,
  "files_total": 27,
  "modules": [
    {
      "name": "helpers",
      "has_spec": false
    }
  ]
}
```

---

## Examples

```bash
# Basic validation
specsync check

# Strict mode — warnings fail the build
specsync check --strict

# Enforce full spec coverage
specsync check --strict --require-coverage 100

# JSON output for tooling
specsync check --json

# Override project root
specsync check --root ./packages/backend

# Watch mode during development
specsync watch

# See what needs specs
specsync coverage

# Bootstrap specs for existing code
specsync generate
```
